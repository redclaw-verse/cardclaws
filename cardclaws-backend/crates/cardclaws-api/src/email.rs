//! Transactional email behind an [`EmailSender`] trait. Production uses Resend
//! (PRD §7.2); tests use [`CapturingEmailSender`] to assert on what was sent.

use std::sync::Mutex;

use async_trait::async_trait;
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[error("email send failed: {0}")]
pub struct EmailError(pub String);

#[async_trait]
pub trait EmailSender: Send + Sync {
    async fn send(&self, to: &str, subject: &str, html: &str) -> Result<(), EmailError>;
}

// ---- Resend ---------------------------------------------------------------

pub struct ResendEmailSender {
    client: reqwest::Client,
    api_key: String,
    from: String,
}

impl ResendEmailSender {
    pub fn new(api_key: String, from: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            from,
        }
    }
}

#[derive(Serialize)]
struct ResendPayload<'a> {
    from: &'a str,
    to: [&'a str; 1],
    subject: &'a str,
    html: &'a str,
}

#[async_trait]
impl EmailSender for ResendEmailSender {
    async fn send(&self, to: &str, subject: &str, html: &str) -> Result<(), EmailError> {
        let resp = self
            .client
            .post("https://api.resend.com/emails")
            .bearer_auth(&self.api_key)
            .json(&ResendPayload {
                from: &self.from,
                to: [to],
                subject,
                html,
            })
            .send()
            .await
            .map_err(|e| EmailError(e.to_string()))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(EmailError(format!("resend status {}", resp.status())))
        }
    }
}

// ---- Capturing fake (tests) ----------------------------------------------

#[derive(Default)]
pub struct CapturingEmailSender {
    pub sent: Mutex<Vec<SentEmail>>,
}

#[derive(Debug, Clone)]
pub struct SentEmail {
    pub to: String,
    pub subject: String,
    pub html: String,
}

#[async_trait]
impl EmailSender for CapturingEmailSender {
    async fn send(&self, to: &str, subject: &str, html: &str) -> Result<(), EmailError> {
        self.sent.lock().unwrap().push(SentEmail {
            to: to.to_string(),
            subject: subject.to_string(),
            html: html.to_string(),
        });
        Ok(())
    }
}
