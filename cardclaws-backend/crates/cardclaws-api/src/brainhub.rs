//! ClawBrainHub (clawbrainhub.com) integration: list agent "brains" and pull a
//! brain's identity/skills so the app can turn it into a trading-card-style
//! agent card. Reads use the public discovery API (no auth); an optional token
//! is sent as a Bearer for the user's private brains.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const DEFAULT_BASE: &str = "https://clawbrainhub.com/api/v1";
/// Reference brains the registry always ships — handy picks even when the user
/// hasn't published their own yet.
const REFERENCE_OWNER: &str = "redclawsystems";

/// A brain as shown in the picker list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BrainSummary {
    pub owner: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub trust_score: Option<i64>,
    pub badge: Option<String>,
}

/// A pulled brain normalized for a card (identity + skills/tools/capabilities).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentBrain {
    pub owner: String,
    pub name: String,
    pub version: String,
    pub tagline: String,
    pub skills: Vec<String>,
    pub tools: Vec<String>,
    pub capabilities: Vec<String>,
}

#[derive(Debug)]
pub enum BrainHubError {
    Http(String),
    Parse(String),
}

impl std::fmt::Display for BrainHubError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrainHubError::Http(e) => write!(f, "clawbrainhub request failed: {e}"),
            BrainHubError::Parse(e) => write!(f, "clawbrainhub parse failed: {e}"),
        }
    }
}

#[async_trait]
pub trait BrainHubClient: Send + Sync {
    /// The user's brains plus the registry's reference brains.
    async fn list_brains(&self) -> Result<Vec<BrainSummary>, BrainHubError>;
    /// Pull a brain and normalize it into card fields.
    async fn pull_brain(
        &self,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<AgentBrain, BrainHubError>;
}

pub struct HttpBrainHubClient {
    http: reqwest::Client,
    base: String,
    /// The user's ClawBrainHub account (its brains are listed first).
    owner: String,
    token: Option<String>,
}

impl HttpBrainHubClient {
    pub fn new(owner: String, token: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base: DEFAULT_BASE.to_string(),
            owner,
            token,
        }
    }

    async fn get_json(&self, url: &str) -> Result<serde_json::Value, BrainHubError> {
        let mut req = self.http.get(url);
        if let Some(t) = &self.token {
            req = req.bearer_auth(t);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| BrainHubError::Http(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(BrainHubError::Http(format!("status {}", resp.status())));
        }
        resp.json()
            .await
            .map_err(|e| BrainHubError::Parse(e.to_string()))
    }

    async fn list_owner(&self, owner: &str) -> Vec<BrainSummary> {
        let url = format!("{}/brains/{}", self.base, owner);
        match self.get_json(&url).await {
            Ok(json) => parse_summaries(&json),
            Err(_) => Vec::new(),
        }
    }
}

#[async_trait]
impl BrainHubClient for HttpBrainHubClient {
    async fn list_brains(&self) -> Result<Vec<BrainSummary>, BrainHubError> {
        let mut out = self.list_owner(&self.owner).await;
        for b in self.list_owner(REFERENCE_OWNER).await {
            if !out.iter().any(|x| x.owner == b.owner && x.name == b.name) {
                out.push(b);
            }
        }
        Ok(out)
    }

    async fn pull_brain(
        &self,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<AgentBrain, BrainHubError> {
        let url = format!("{}/brains/{}/{}/{}/pull", self.base, owner, name, version);
        let json = self.get_json(&url).await?;
        Ok(parse_brain(owner, name, version, &json))
    }
}

/// Parse a `/brains/{owner}` list response into summaries (latest version each).
fn parse_summaries(json: &serde_json::Value) -> Vec<BrainSummary> {
    let arr = match json.as_array() {
        Some(a) => a,
        None => return Vec::new(),
    };
    arr.iter()
        .filter_map(|b| {
            let owner = b.get("owner")?.as_str()?.to_string();
            let name = b.get("name")?.as_str()?.to_string();
            // First non-yanked version (the API lists newest first).
            let ver = b.get("versions").and_then(|v| v.as_array()).and_then(|vs| {
                vs.iter()
                    .find(|v| !v.get("yanked").and_then(|y| y.as_bool()).unwrap_or(false))
            });
            let version = ver
                .and_then(|v| v.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("latest")
                .to_string();
            Some(BrainSummary {
                owner,
                name,
                description: b
                    .get("description")
                    .and_then(|d| d.as_str())
                    .map(str::to_string),
                version,
                trust_score: ver
                    .and_then(|v| v.get("trust_score"))
                    .and_then(|t| t.as_i64()),
                badge: ver
                    .and_then(|v| v.get("badge"))
                    .and_then(|x| x.as_str())
                    .map(str::to_string),
            })
        })
        .collect()
}

/// Normalize a pulled `.brain` JSON into card fields.
fn parse_brain(owner: &str, name: &str, version: &str, json: &serde_json::Value) -> AgentBrain {
    let meta_name = json
        .pointer("/meta/brain_name")
        .and_then(|v| v.as_str())
        .unwrap_or(name);
    let agent_md = json
        .pointer("/identity/agent_md")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let soul_md = json
        .pointer("/identity/soul_md")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let skills_md = json
        .pointer("/skills/skills_md")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut skills = md_headings(skills_md);
    skills.truncate(8);

    let mut tools: Vec<String> = json
        .pointer("/skills/tool_defs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    tools.truncate(8);

    let mut capabilities = md_bullets(agent_md);
    if capabilities.is_empty() {
        capabilities = md_headings(agent_md)
            .into_iter()
            .filter(|h| h != "Purpose")
            .collect();
    }
    capabilities.truncate(6);

    let tagline = first_paragraph(soul_md)
        .or_else(|| first_paragraph(agent_md))
        .unwrap_or_default();

    AgentBrain {
        owner: owner.to_string(),
        name: meta_name.to_string(),
        version: version.to_string(),
        tagline,
        skills,
        tools,
        capabilities,
    }
}

/// `## Heading` lines, prettified (snake_case → Title Case).
fn md_headings(md: &str) -> Vec<String> {
    md.lines()
        .filter_map(|l| l.trim().strip_prefix("## "))
        .map(|h| {
            h.trim()
                .split(['_', '-', ' '])
                .filter(|w| !w.is_empty())
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        None => String::new(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect()
}

/// `- ` / `* ` bullet lines, stripped.
fn md_bullets(md: &str) -> Vec<String> {
    md.lines()
        .filter_map(|l| {
            let t = l.trim();
            t.strip_prefix("- ")
                .or_else(|| t.strip_prefix("* "))
                .map(|s| s.trim().to_string())
        })
        .filter(|s| !s.is_empty() && s.len() < 80)
        .collect()
}

/// First non-empty, non-heading, non-bullet line.
fn first_paragraph(md: &str) -> Option<String> {
    md.lines()
        .map(str::trim)
        .find(|l| {
            !l.is_empty() && !l.starts_with('#') && !l.starts_with('-') && !l.starts_with('*')
        })
        .map(|s| {
            if s.chars().count() <= 150 {
                return s.to_string();
            }
            // Truncate at a word boundary so the tagline doesn't end mid-word.
            let head: String = s.chars().take(150).collect();
            let cut = head.rsplit_once(' ').map(|(a, _)| a).unwrap_or(&head);
            format!("{}…", cut.trim_end_matches([',', '.', ';', ':', ' ']))
        })
}

/// Deterministic double for tests.
pub struct FakeBrainHubClient;

#[async_trait]
impl BrainHubClient for FakeBrainHubClient {
    async fn list_brains(&self) -> Result<Vec<BrainSummary>, BrainHubError> {
        Ok(vec![BrainSummary {
            owner: "redclawsystems".into(),
            name: "general-assistant".into(),
            description: Some("General-purpose assistant".into()),
            version: "1.0.0".into(),
            trust_score: Some(90),
            badge: Some("Verified".into()),
        }])
    }

    async fn pull_brain(
        &self,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<AgentBrain, BrainHubError> {
        Ok(AgentBrain {
            owner: owner.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            tagline: "A general-purpose assistant.".into(),
            skills: vec!["Web Search".into(), "Code".into()],
            tools: vec![],
            capabilities: vec!["Writing".into(), "Analysis".into()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_skills_and_tagline_from_brain_json() {
        let json = serde_json::json!({
            "meta": { "brain_name": "general-assistant" },
            "identity": {
                "soul_md": "# Soul\n\nI am a helpful general-purpose assistant.",
                "agent_md": "# Agent\n\n## Purpose\n\n- Writing\n- Planning"
            },
            "skills": { "skills_md": "# Skills\n\n## web_search\nSearch the web.\n## code_review\nReview code." }
        });
        let b = parse_brain("redclawsystems", "general-assistant", "1.0.0", &json);
        assert_eq!(b.name, "general-assistant");
        assert_eq!(b.skills, vec!["Web Search", "Code Review"]);
        assert_eq!(b.capabilities, vec!["Writing", "Planning"]);
        assert!(b.tagline.contains("general-purpose assistant"));
    }

    #[test]
    fn parses_list_summaries() {
        let json = serde_json::json!([
            { "owner": "o", "name": "n", "description": null,
              "versions": [{ "version": "1.0.0", "trust_score": 85, "badge": "Verified", "yanked": false }] }
        ]);
        let s = parse_summaries(&json);
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].version, "1.0.0");
        assert_eq!(s[0].trust_score, Some(85));
    }
}
