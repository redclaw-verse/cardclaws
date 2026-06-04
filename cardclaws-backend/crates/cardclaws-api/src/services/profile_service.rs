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
