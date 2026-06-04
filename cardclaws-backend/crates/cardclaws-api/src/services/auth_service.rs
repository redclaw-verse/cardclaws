//! Authentication business logic. Handlers stay thin (PRD §9.2) and delegate
//! here. This module wires the `cardclaws-auth` primitives to the database and
//! cache.

use chrono::{Duration, Utc};
use rand::Rng;

use cardclaws_auth::token::{generate_opaque_token, hash_token};
use cardclaws_auth::{apple, jwt, magic_link, password};
use cardclaws_db::queries::{sessions, users};
use cardclaws_types::auth::*;
use cardclaws_types::{AppError, User};

use crate::error::SqlxResultExt;
use crate::middleware::rate_limit;
use crate::state::AppState;

/// Refresh-token lifetime (30 days, PRD §6.8.1).
const REFRESH_TTL_DAYS: i64 = 30;

// ---- Public flows ---------------------------------------------------------

pub async fn register(state: &AppState, req: RegisterRequest) -> Result<TokenPair, AppError> {
    crate::validation::validate_email(&req.email)?;
    crate::validation::validate_password(&req.password)?;
    crate::validation::validate_handle(&req.handle)?;

    if users::email_or_handle_taken(&state.db, &req.email, &req.handle)
        .await
        .map_db()?
    {
        return Err(AppError::Conflict("email or handle already in use".into()));
    }

    let hash = password::hash_password(&req.password)
        .map_err(|_| AppError::Internal("password hashing failed".into()))?;

    let user = users::insert(
        &state.db,
        users::NewUser {
            email: &req.email,
            handle: &req.handle,
            display_name: &req.display_name,
            password_hash: Some(&hash),
        },
    )
    .await
    .map_err(map_unique_violation)?;

    issue_tokens(state, &user, None).await
}

pub async fn login(
    state: &AppState,
    req: LoginRequest,
    device_fingerprint: Option<&str>,
) -> Result<TokenPair, AppError> {
    // Lockout: 10 failed attempts / 5 min per email (§20.1).
    let lock_key = format!("login:fail:{}", req.email);

    let user = users::find_by_email(&state.db, &req.email).await.map_db()?;
    let Some(user) = user else {
        // Count the attempt even for unknown emails to avoid timing/enumeration.
        bump_login_failures(state, &lock_key).await?;
        return Err(AppError::Unauthorized);
    };

    let Some(stored) = &user.password_hash else {
        // OAuth/magic-link-only account: no password to check.
        return Err(AppError::Unauthorized);
    };

    let ok = password::verify_password(&req.password, stored).unwrap_or(false);
    if !ok {
        bump_login_failures(state, &lock_key).await?;
        return Err(AppError::Unauthorized);
    }

    issue_tokens(state, &user, device_fingerprint).await
}

/// Always returns Ok(()) regardless of whether the email exists, to avoid
/// account enumeration. Only sends a link if the account is real.
pub async fn magic_link_request(state: &AppState, req: MagicLinkRequest) -> Result<(), AppError> {
    crate::validation::validate_email(&req.email)?;

    if let Some(user) = users::find_by_email(&state.db, &req.email).await.map_db()? {
        let minted = magic_link::mint();
        let key = format!("magic:{}", minted.hash);
        state
            .cache
            .set_ex(&key, &user.email, magic_link::MAGIC_LINK_TTL_SECS)
            .await
            .map_err(|e| AppError::Internal(e.0))?;

        let link = format!(
            "{}/auth/magic?token={}",
            state.profile_base_url, minted.token
        );
        let html = format!(
            "<p>Tap to sign in to CardClaws:</p><p><a href=\"{link}\">Sign in</a></p>\
             <p>This link expires in 15 minutes.</p>"
        );
        // Best-effort: a transient email failure should not leak account state.
        let _ = state
            .email
            .send(&user.email, "Your CardClaws sign-in link", &html)
            .await;
    }
    Ok(())
}

pub async fn magic_link_verify(
    state: &AppState,
    req: MagicLinkVerifyRequest,
) -> Result<TokenPair, AppError> {
    let hash = hash_token(&req.token);
    let key = format!("magic:{hash}");
    let email = state
        .cache
        .get_del(&key)
        .await
        .map_err(|e| AppError::Internal(e.0))?
        .ok_or(AppError::Unauthorized)?;

    let user = users::find_by_email(&state.db, &email)
        .await
        .map_db()?
        .ok_or(AppError::Unauthorized)?;

    issue_tokens(state, &user, None).await
}

pub async fn oauth_apple(state: &AppState, req: AppleOAuthRequest) -> Result<TokenPair, AppError> {
    let claims = apple::verify_identity_token(
        state.apple.as_ref(),
        &req.identity_token,
        &state.apple_audience,
    )
    .await
    .map_err(|_| AppError::Unauthorized)?;

    let email = claims.email.ok_or_else(|| {
        AppError::BadRequest("Apple did not provide an email for this account".into())
    })?;

    if let Some(user) = users::find_by_email(&state.db, &email).await.map_db()? {
        return issue_tokens(state, &user, None).await;
    }

    // First sign-in: provision an account with a generated unique handle.
    let display_name = req
        .display_name
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| email.split('@').next().unwrap_or("user").to_string());
    let handle = generate_unique_handle(state, &email).await?;

    let user = users::insert(
        &state.db,
        users::NewUser {
            email: &email,
            handle: &handle,
            display_name: &display_name,
            password_hash: None,
        },
    )
    .await
    .map_err(map_unique_violation)?;

    issue_tokens(state, &user, None).await
}

pub async fn refresh(state: &AppState, req: RefreshRequest) -> Result<TokenPair, AppError> {
    let hash = hash_token(&req.refresh_token);
    let session = sessions::find_live_by_hash(&state.db, &hash)
        .await
        .map_db()?
        .ok_or(AppError::Unauthorized)?;

    let user = users::find_by_id(&state.db, session.user_id)
        .await
        .map_db()?
        .ok_or(AppError::Unauthorized)?;

    // Rotate: invalidate the presented token, then mint a fresh pair.
    sessions::delete(&state.db, session.id).await.map_db()?;
    issue_tokens(state, &user, session.device_fingerprint.as_deref()).await
}

pub async fn logout(state: &AppState, refresh_token: &str) -> Result<(), AppError> {
    let hash = hash_token(refresh_token);
    if let Some(session) = sessions::find_live_by_hash(&state.db, &hash)
        .await
        .map_db()?
    {
        sessions::delete(&state.db, session.id).await.map_db()?;
    }
    Ok(())
}

// ---- Helpers --------------------------------------------------------------

/// Mint an access JWT + a fresh refresh token, persisting the refresh session.
async fn issue_tokens(
    state: &AppState,
    user: &User,
    device_fingerprint: Option<&str>,
) -> Result<TokenPair, AppError> {
    let access_token = state
        .jwt
        .encode_access(user.id, user.tier)
        .map_err(|_| AppError::Internal("token encoding failed".into()))?;

    let refresh_token = generate_opaque_token();
    let refresh_hash = hash_token(&refresh_token);
    let expires_at = Utc::now() + Duration::days(REFRESH_TTL_DAYS);

    sessions::insert(
        &state.db,
        sessions::NewSession {
            user_id: user.id,
            refresh_token_hash: &refresh_hash,
            device_fingerprint,
            expires_at,
        },
    )
    .await
    .map_db()?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        expires_in: jwt::ACCESS_TOKEN_TTL_SECS,
        user: AuthUserInfo {
            id: user.id,
            email: user.email.clone(),
            handle: user.handle.clone(),
            display_name: user.display_name.clone(),
            tier: user.tier,
        },
    })
}

async fn bump_login_failures(state: &AppState, key: &str) -> Result<(), AppError> {
    let count = rate_limit::check(
        state.cache.as_ref(),
        key,
        rate_limit::LOGIN_MAX_FAILS,
        rate_limit::LOGIN_WINDOW_SECS,
    )
    .await;
    // `check` returns RateLimited once the threshold is crossed; surface that as
    // an explicit lockout so the client can message it.
    match count {
        Err(AppError::RateLimited) => Err(AppError::RateLimited),
        _ => Ok(()),
    }
}

/// Build a unique handle from an email local part plus random digits on
/// collision. Sanitizes to the handle charset and pads short names.
async fn generate_unique_handle(state: &AppState, email: &str) -> Result<String, AppError> {
    let mut base: String = email
        .split('@')
        .next()
        .unwrap_or("user")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .map(|c| c.to_ascii_lowercase())
        .collect();
    if base.chars().count() < 3 {
        base = format!("user{base}");
    }
    base.truncate(24);

    if !users::handle_taken(&state.db, &base).await.map_db()? && !is_reserved(&base) {
        return Ok(base);
    }
    for _ in 0..10 {
        let suffix: u32 = rand::thread_rng().gen_range(1000..9999);
        let candidate = format!("{base}{suffix}");
        if !users::handle_taken(&state.db, &candidate).await.map_db()? {
            return Ok(candidate);
        }
    }
    Err(AppError::Internal(
        "could not allocate a unique handle".into(),
    ))
}

fn is_reserved(handle: &str) -> bool {
    crate::validation::validate_handle(handle).is_err()
}

/// Map a Postgres unique-violation into a clean 409.
fn map_unique_violation(e: sqlx::Error) -> AppError {
    if let sqlx::Error::Database(db_err) = &e {
        if db_err.is_unique_violation() {
            return AppError::Conflict("email or handle already in use".into());
        }
    }
    AppError::Internal(format!("db: {e}"))
}
