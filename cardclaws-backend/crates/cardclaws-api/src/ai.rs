//! AI image generation via Google Gemini (PRD §21 Phase 5 "Card AI Generator").
//!
//! Two steps: refine the user's rough fields into a single vivid prompt
//! (gemini-2.5-flash), then generate the image (gemini-2.5-flash-image, aka
//! "nano-banana"). The API key lives only here on the server — loaded from
//! Infisical via config — and never reaches the client.
//!
//! Behind an [`AiClient`] trait so handlers/tests don't depend on the network.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const GEMINI_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";
const TEXT_MODEL: &str = "gemini-2.5-flash";
const IMAGE_MODEL: &str = "gemini-2.5-flash-image"; // "nano-banana"
const VIDEO_MODEL: &str = "veo-2.0-generate-001"; // Google Veo (async)

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("AI is not configured")]
    NotConfigured,
    #[error("gemini request failed: {0}")]
    Request(String),
    #[error("gemini returned no {0}")]
    Empty(&'static str),
}

/// The user's persona (from their onboarding profile) used to tailor generation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Persona {
    pub role: Option<String>,
    pub vibe: Option<String>,
    pub colors: Option<Vec<String>>,
    pub goal: Option<String>,
}

/// Rough fields collected by the "Describe your scene" wizard.
#[derive(Debug, Clone, Serialize)]
pub struct SceneBrief {
    pub scene: String,
    pub style: Option<String>,
    pub mood: Option<String>,
    pub persona: Option<Persona>,
}

/// A generated image as base64 plus its MIME type.
pub struct GeneratedImage {
    pub mime_type: String,
    pub base64: String,
}

/// State of an async Veo video job.
pub enum VideoStatus {
    Pending,
    Done(Vec<u8>),
    Failed(String),
}

#[async_trait]
pub trait AiClient: Send + Sync {
    /// Refine the brief into a single image-generation prompt.
    async fn refine_prompt(&self, brief: &SceneBrief) -> Result<String, AiError>;
    /// Generate an image from a prompt (nano-banana).
    async fn generate_image(&self, prompt: &str) -> Result<GeneratedImage, AiError>;
    /// Submit a Veo video job; returns the long-running operation name.
    async fn start_video(&self, prompt: &str) -> Result<String, AiError>;
    /// Poll a Veo operation; downloads + returns the MP4 bytes when done.
    async fn poll_video(&self, operation: &str) -> Result<VideoStatus, AiError>;
}

// ---- Gemini (production) --------------------------------------------------

pub struct GeminiClient {
    api_key: String,
    http: reqwest::Client,
}

impl GeminiClient {
    /// Returns `None` when no key is configured (AI stays disabled).
    pub fn new(api_key: String) -> Option<Self> {
        if api_key.trim().is_empty() {
            return None;
        }
        Some(Self {
            api_key,
            http: reqwest::Client::new(),
        })
    }

    async fn generate_content(
        &self,
        model: &str,
        text: &str,
    ) -> Result<serde_json::Value, AiError> {
        let url = format!("{GEMINI_BASE}/models/{model}:generateContent");
        let body = serde_json::json!({
            "contents": [{ "parts": [{ "text": text }] }]
        });
        let resp = self
            .http
            .post(&url)
            .query(&[("key", &self.api_key)])
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Request(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AiError::Request(format!("status {}", resp.status())));
        }
        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| AiError::Request(e.to_string()))
    }
}

#[async_trait]
impl AiClient for GeminiClient {
    async fn refine_prompt(&self, brief: &SceneBrief) -> Result<String, AiError> {
        let instruction = format!(
            "You are helping design a digital business card image. Turn the \
             following into ONE vivid, concrete image-generation prompt (2-3 \
             sentences, no preamble, no quotes).\nScene: {}\nStyle: {}\nMood: {}{}",
            brief.scene,
            brief.style.as_deref().unwrap_or("(any)"),
            brief.mood.as_deref().unwrap_or("(any)"),
            persona_clause(brief.persona.as_ref()),
        );
        let json = self.generate_content(TEXT_MODEL, &instruction).await?;
        first_text(&json).ok_or(AiError::Empty("text"))
    }

    async fn generate_image(&self, prompt: &str) -> Result<GeneratedImage, AiError> {
        let json = self.generate_content(IMAGE_MODEL, prompt).await?;
        first_inline_image(&json).ok_or(AiError::Empty("image"))
    }

    async fn start_video(&self, prompt: &str) -> Result<String, AiError> {
        let url = format!("{GEMINI_BASE}/models/{VIDEO_MODEL}:predictLongRunning");
        let body = serde_json::json!({
            "instances": [{ "prompt": prompt }],
            "parameters": { "aspectRatio": "9:16", "durationSeconds": 5 }
        });
        let resp = self
            .http
            .post(&url)
            .query(&[("key", &self.api_key)])
            .json(&body)
            .send()
            .await
            .map_err(|e| AiError::Request(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AiError::Request(format!("status {}", resp.status())));
        }
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AiError::Request(e.to_string()))?;
        json["name"]
            .as_str()
            .map(str::to_string)
            .ok_or(AiError::Empty("operation"))
    }

    async fn poll_video(&self, operation: &str) -> Result<VideoStatus, AiError> {
        let url = format!("{GEMINI_BASE}/{operation}");
        let json: serde_json::Value = self
            .http
            .get(&url)
            .query(&[("key", &self.api_key)])
            .send()
            .await
            .map_err(|e| AiError::Request(e.to_string()))?
            .json()
            .await
            .map_err(|e| AiError::Request(e.to_string()))?;

        if let Some(err) = json.get("error") {
            let msg = err["message"].as_str().unwrap_or("video generation failed");
            return Ok(VideoStatus::Failed(msg.to_string()));
        }
        if !json["done"].as_bool().unwrap_or(false) {
            return Ok(VideoStatus::Pending);
        }
        // Done: the sample carries a download URI (302-redirects to a signed
        // URL; reqwest follows it). The key authorizes the first hop.
        let uri = json
            .pointer("/response/generateVideoResponse/generatedSamples/0/video/uri")
            .and_then(|v| v.as_str())
            .ok_or(AiError::Empty("video"))?;
        let bytes = self
            .http
            .get(uri)
            .query(&[("key", &self.api_key)])
            .send()
            .await
            .map_err(|e| AiError::Request(e.to_string()))?
            .error_for_status()
            .map_err(|e| AiError::Request(e.to_string()))?
            .bytes()
            .await
            .map_err(|e| AiError::Request(e.to_string()))?;
        Ok(VideoStatus::Done(bytes.to_vec()))
    }
}

/// Build the persona suffix for the refine instruction (empty when no persona).
fn persona_clause(persona: Option<&Persona>) -> String {
    match persona {
        None => String::new(),
        Some(p) => format!(
            "\nPersona (bias the image to fit this person): role={}; aesthetic vibe={}; \
             brand colors={}; goal={}. Weave the brand colors and vibe into the composition; \
             do not render any text.",
            p.role.as_deref().unwrap_or("(any)"),
            p.vibe.as_deref().unwrap_or("(any)"),
            p.colors.as_ref().map(|c| c.join(", ")).unwrap_or_default(),
            p.goal.as_deref().unwrap_or("(any)"),
        ),
    }
}

/// Extract the first text part from a generateContent response.
fn first_text(json: &serde_json::Value) -> Option<String> {
    json["candidates"][0]["content"]["parts"]
        .as_array()?
        .iter()
        .find_map(|p| p["text"].as_str())
        .map(|s| s.trim().to_string())
}

/// Extract the first inline image part (base64 + mime) from a response.
fn first_inline_image(json: &serde_json::Value) -> Option<GeneratedImage> {
    json["candidates"][0]["content"]["parts"]
        .as_array()?
        .iter()
        .find_map(|p| {
            let data = p["inlineData"]["data"].as_str()?;
            let mime = p["inlineData"]["mimeType"].as_str().unwrap_or("image/png");
            Some(GeneratedImage {
                mime_type: mime.to_string(),
                base64: data.to_string(),
            })
        })
}

/// Used when no API key is configured — every call errors `NotConfigured`.
pub struct DisabledAiClient;

#[async_trait]
impl AiClient for DisabledAiClient {
    async fn refine_prompt(&self, _brief: &SceneBrief) -> Result<String, AiError> {
        Err(AiError::NotConfigured)
    }
    async fn generate_image(&self, _prompt: &str) -> Result<GeneratedImage, AiError> {
        Err(AiError::NotConfigured)
    }
    async fn start_video(&self, _prompt: &str) -> Result<String, AiError> {
        Err(AiError::NotConfigured)
    }
    async fn poll_video(&self, _operation: &str) -> Result<VideoStatus, AiError> {
        Err(AiError::NotConfigured)
    }
}

// ---- Fake (tests) ---------------------------------------------------------

/// Deterministic fake: echoes a refined prompt and returns a 1x1 PNG.
pub struct FakeAiClient;

const PIXEL_PNG_B64: &str =
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

#[async_trait]
impl AiClient for FakeAiClient {
    async fn refine_prompt(&self, brief: &SceneBrief) -> Result<String, AiError> {
        let role = brief
            .persona
            .as_ref()
            .and_then(|p| p.role.as_deref())
            .unwrap_or("");
        Ok(format!(
            "A {} {} scene for {}: {}",
            brief.mood.as_deref().unwrap_or("striking"),
            brief.style.as_deref().unwrap_or("cinematic"),
            role,
            brief.scene,
        ))
    }

    async fn generate_image(&self, _prompt: &str) -> Result<GeneratedImage, AiError> {
        Ok(GeneratedImage {
            mime_type: "image/png".to_string(),
            base64: PIXEL_PNG_B64.to_string(),
        })
    }
    async fn start_video(&self, _prompt: &str) -> Result<String, AiError> {
        Ok("models/veo-2.0-generate-001/operations/fake".to_string())
    }
    async fn poll_video(&self, _operation: &str) -> Result<VideoStatus, AiError> {
        // Minimal valid MP4 ftyp header — enough for the endpoint contract.
        Ok(VideoStatus::Done(vec![
            0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6f, 0x6d,
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_and_image_from_gemini_shape() {
        let text_resp = serde_json::json!({
            "candidates": [{ "content": { "parts": [{ "text": "  a vivid prompt  " }] } }]
        });
        assert_eq!(first_text(&text_resp).as_deref(), Some("a vivid prompt"));

        let img_resp = serde_json::json!({
            "candidates": [{ "content": { "parts": [
                { "text": "here is your image" },
                { "inlineData": { "mimeType": "image/png", "data": "QUJD" } }
            ] } }]
        });
        let img = first_inline_image(&img_resp).unwrap();
        assert_eq!(img.mime_type, "image/png");
        assert_eq!(img.base64, "QUJD");
    }

    #[test]
    fn disabled_without_key() {
        assert!(GeminiClient::new("".into()).is_none());
        assert!(GeminiClient::new("   ".into()).is_none());
        assert!(GeminiClient::new("real-key".into()).is_some());
    }
}
