//! Lightweight field validation shared by handlers. Mirrors the mobile
//! `handleValidation` rules so client and server agree (PRD §6.6.1, §6.8.3).

use cardclaws_types::AppError;

/// Handles: 3-30 chars, lowercase alphanumeric and hyphen, no leading/trailing
/// hyphen. Reserved handles are rejected.
pub fn validate_handle(handle: &str) -> Result<(), AppError> {
    let len = handle.chars().count();
    if !(3..=30).contains(&len) {
        return Err(AppError::Validation(
            "handle must be 3-30 characters".into(),
        ));
    }
    if handle.starts_with('-') || handle.ends_with('-') {
        return Err(AppError::Validation(
            "handle cannot start or end with a hyphen".into(),
        ));
    }
    if !handle
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Err(AppError::Validation(
            "handle may only contain lowercase letters, digits, and hyphens".into(),
        ));
    }
    if is_reserved(handle) {
        return Err(AppError::Conflict("handle is reserved".into()));
    }
    Ok(())
}

const RESERVED: &[&str] = &[
    "admin",
    "support",
    "cardclaws",
    "help",
    "api",
    "www",
    "app",
    "about",
    "login",
    "logout",
    "register",
    "settings",
    "profile",
    "s",
    "v1",
    "assets",
];

fn is_reserved(handle: &str) -> bool {
    // All two-character strings are reserved (§6.8.3); the length check already
    // rejects <3, so we only need the explicit list here.
    RESERVED.contains(&handle)
}

pub fn validate_email(email: &str) -> Result<(), AppError> {
    // Deliberately permissive: exactly one '@', non-empty local + domain, and a
    // dot in the domain. Full RFC 5322 validation is not worth the surface.
    let parts: Vec<&str> = email.split('@').collect();
    let ok = parts.len() == 2
        && !parts[0].is_empty()
        && parts[1].contains('.')
        && !parts[1].starts_with('.')
        && !parts[1].ends_with('.');
    if ok {
        Ok(())
    } else {
        Err(AppError::Validation("invalid email address".into()))
    }
}

pub fn validate_password(password: &str) -> Result<(), AppError> {
    if password.chars().count() < 8 {
        return Err(AppError::Validation(
            "password must be at least 8 characters".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_handles_pass() {
        for h in ["omar", "red-claw", "card123", "a1b"] {
            assert!(validate_handle(h).is_ok(), "{h} should be valid");
        }
    }

    #[test]
    fn invalid_handles_fail() {
        for h in ["ab", "-omar", "omar-", "Omar", "om ar", "admin", "api"] {
            assert!(validate_handle(h).is_err(), "{h} should be invalid");
        }
    }

    #[test]
    fn email_validation() {
        assert!(validate_email("a@b.com").is_ok());
        assert!(validate_email("bad").is_err());
        assert!(validate_email("a@b").is_err());
        assert!(validate_email("a@@b.com").is_err());
    }

    #[test]
    fn password_min_length() {
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("short").is_err());
    }
}
