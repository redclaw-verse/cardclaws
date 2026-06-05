//! Public profile actions — currently the contact form (PRD §11.4). A visitor
//! submits name/email/message; we validate, record an analytics event, and email
//! the card owner. Always rate-limited per handle to prevent abuse.

use cardclaws_db::queries::users;
use cardclaws_types::AppError;

use crate::error::SqlxResultExt;
use crate::middleware::rate_limit;
use crate::services::{analytics_service, card_service};
use crate::state::AppState;

pub async fn submit_contact(
    state: &AppState,
    handle: &str,
    name: &str,
    email: &str,
    message: &str,
) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }
    crate::validation::validate_email(email)?;
    if message.trim().is_empty() || message.chars().count() > 1000 {
        return Err(AppError::Validation(
            "message must be 1–1000 characters".into(),
        ));
    }

    // 5 submissions / minute / handle.
    rate_limit::check(state.cache.as_ref(), &format!("contact:{handle}"), 5, 60).await?;

    let card = card_service::get_public_by_handle(state, handle).await?;
    let owner = users::find_by_id(&state.db, card.owner_id)
        .await
        .map_db()?
        .ok_or_else(|| AppError::Internal("card owner missing".into()))?;

    // Record the submission (best-effort) and email the owner.
    let _ = analytics_service::record(state, card.id, "contact_form_submission", None, None, None)
        .await;

    let html = format!(
        "<p>New message via your CardClaws profile from <strong>{}</strong> ({}):</p><blockquote>{}</blockquote>",
        escape_html(name),
        escape_html(email),
        escape_html(message),
    );
    let _ = state
        .email
        .send(&owner.email, "New CardClaws contact form message", &html)
        .await;

    Ok(())
}

/// "We Met": a scanner shares back; capture the connection (with coarse geo) for
/// the card owner's "People I met" list, and email the owner. Rate-limited.
pub async fn submit_connection(
    state: &AppState,
    handle: &str,
    name: &str,
    email: Option<&str>,
    note: Option<&str>,
    ip: Option<&str>,
) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }
    if let Some(e) = email {
        if !e.trim().is_empty() {
            crate::validation::validate_email(e)?;
        }
    }

    rate_limit::check(state.cache.as_ref(), &format!("connect:{handle}"), 5, 60).await?;

    let card = card_service::get_public_by_handle(state, handle).await?;

    // Resolve coarse geo, then drop the raw IP (PRD §18.3).
    let geo = ip.map(|raw| state.geo.resolve(raw));
    let country = geo.as_ref().and_then(|g| g.country.as_deref());
    let city = geo.as_ref().and_then(|g| g.city.as_deref());

    cardclaws_db::queries::connections::insert(
        &state.db,
        cardclaws_db::queries::connections::NewConnection {
            card_id: card.id,
            owner_id: card.owner_id,
            name: name.trim(),
            email: email.map(str::trim).filter(|e| !e.is_empty()),
            note: note.map(str::trim).filter(|n| !n.is_empty()),
            country,
            city,
        },
    )
    .await
    .map_db()?;

    // Notify the owner (best-effort).
    if let Some(owner) = users::find_by_id(&state.db, card.owner_id).await.map_db()? {
        let where_ = city
            .map(|c| format!(" in {}", escape_html(c)))
            .unwrap_or_default();
        let note_html = note
            .filter(|n| !n.trim().is_empty())
            .map(|n| format!("<blockquote>{}</blockquote>", escape_html(n)))
            .unwrap_or_default();
        let html = format!(
            "<p><strong>{}</strong> connected with you via CardClaws{}.</p>{}",
            escape_html(name),
            where_,
            note_html,
        );
        let _ = state
            .email
            .send(&owner.email, "New CardClaws connection", &html)
            .await;
    }

    Ok(())
}

/// Minimal HTML escaping so the visitor's text can't inject markup into the email.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::escape_html;

    #[test]
    fn escapes_markup() {
        assert_eq!(escape_html("<b>&hi</b>"), "&lt;b&gt;&amp;hi&lt;/b&gt;");
    }
}
