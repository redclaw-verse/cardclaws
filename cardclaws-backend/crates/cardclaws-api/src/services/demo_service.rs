//! Anonymous "publish to web" for the no-login demo: turns a demo card into a
//! real, scannable public profile (with an optional AI welcome) without auth.
//! Each publish creates a fresh password-less owner so the profile shows the
//! right name and stays within the free-tier 1-card limit.

use cardclaws_db::queries::users;
use cardclaws_types::{AppError, Tier};
use serde::Deserialize;
use uuid::Uuid;

use crate::error::SqlxResultExt;
use crate::services::card_service;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct DemoLink {
    pub label: String,
    pub url: String,
}

/// "Payload drop": the thing handed over on scan — a link CTA or a code.
#[derive(Deserialize)]
pub struct DemoPayload {
    /// "link" | "code".
    pub kind: String,
    pub label: String,
    pub value: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DemoPublishRequest {
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub links: Vec<DemoLink>,
    #[serde(default)]
    pub welcome_prompt: Option<String>,
    #[serde(default)]
    pub welcome_message: Option<String>,
    #[serde(default)]
    pub payload: Option<DemoPayload>,
    /// "Now" status — what the person is currently up to.
    #[serde(default)]
    pub now: Option<String>,
}

pub struct Published {
    pub handle: String,
    pub profile_url: String,
}

pub async fn publish(state: &AppState, req: &DemoPublishRequest) -> Result<Published, AppError> {
    if req.name.trim().is_empty() {
        return Err(AppError::Validation("name is required".into()));
    }

    let suffix: String = Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(10)
        .collect();
    let card_handle = format!("{}-{}", slugify(&req.name), &suffix[..6]);

    // Fresh password-less owner so the profile shows this person's name.
    let user = users::insert(
        &state.db,
        users::NewUser {
            email: &format!("demo-{suffix}@cardclaws.local"),
            handle: &format!("u{suffix}"),
            display_name: req.name.trim(),
            password_hash: None,
        },
    )
    .await
    .map_db()?;

    let links: Vec<serde_json::Value> = req
        .links
        .iter()
        .filter(|l| !l.url.trim().is_empty())
        .map(|l| serde_json::json!({ "label": l.label, "url": l.url }))
        .collect();
    let mut profile = serde_json::json!({ "links": links });
    if let Some(p) = &req.payload {
        if !p.value.trim().is_empty() {
            profile["payload"] = serde_json::json!({
                "kind": p.kind,
                "label": p.label,
                "value": p.value,
            });
        }
    }
    if let Some(now) = req.now.as_deref() {
        if !now.trim().is_empty() {
            profile["now"] = serde_json::json!(now.trim());
        }
    }
    let definition = serde_json::json!({
        "face": { "layers": [], "background": { "type": "solid", "value": "#101014" } },
        "back": {
            "layers": [{
                "id": Uuid::new_v4(),
                "type": "contact",
                "x": 0.1, "y": 0.1, "width": 0.8, "height": 0.2, "opacity": 1, "zIndex": 1,
                "fields": { "title": req.title }
            }],
            "background": { "type": "solid", "value": "#101014" }
        },
        "profile": profile
    });

    let card = card_service::create(state, user.id, &card_handle, &definition).await?;

    if let Some(prompt) = req.welcome_prompt.as_deref() {
        if !prompt.trim().is_empty() {
            card_service::set_welcome(
                state,
                card.id,
                user.id,
                prompt,
                req.welcome_message.as_deref(),
                None,
                None,
            )
            .await?;
        }
    }

    card_service::publish(state, card.id, user.id, Tier::Free).await?;

    Ok(Published {
        profile_url: format!("{}/{}", state.profile_base_url, card.handle),
        handle: card.handle,
    })
}

/// Lowercase, hyphenate, trim to a valid handle stem (validate_handle adds the
/// final rules; the random suffix guarantees uniqueness).
fn slugify(name: &str) -> String {
    let mut out = String::new();
    let mut prev_dash = false;
    for c in name.trim().to_lowercase().chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    let stem: String = out.trim_matches('-').chars().take(20).collect();
    let stem = stem.trim_matches('-').to_string();
    if stem.chars().count() < 2 {
        "card".to_string()
    } else {
        stem
    }
}
