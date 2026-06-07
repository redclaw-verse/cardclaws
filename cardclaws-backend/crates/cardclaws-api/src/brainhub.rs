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
/// Public web base for a shareable brain link.
const WEB_BASE: &str = "https://clawbrainhub.com/brains";

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
    /// Public ClawBrainHub URL to access this brain (shareable via QR).
    pub resource_url: String,
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
        fetch_json(&self.http, &self.token, url).await
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
        let mut summaries = self.list_owner(&self.owner).await;
        for b in self.list_owner(REFERENCE_OWNER).await {
            if !summaries
                .iter()
                .any(|x| x.owner == b.owner && x.name == b.name)
            {
                summaries.push(b);
            }
        }

        // Only surface brains we can actually turn into a card: pull each and
        // keep the ones that parse from a JSON .brain with real content.
        // HDF5-packed brains and server-erroring pulls are filtered out so the
        // picker shows only what works. Pulls run concurrently.
        let mut set = tokio::task::JoinSet::new();
        for (i, s) in summaries.iter().enumerate() {
            let http = self.http.clone();
            let token = self.token.clone();
            let url = format!(
                "{}/brains/{}/{}/{}/pull",
                self.base, s.owner, s.name, s.version
            );
            let (owner, name, version) = (s.owner.clone(), s.name.clone(), s.version.clone());
            set.spawn(async move {
                let usable = match fetch_bytes(&http, &token, &url).await {
                    Ok(bytes) => match brain_from_blob(&owner, &name, &version, bytes) {
                        Ok(b) => {
                            !b.skills.is_empty()
                                || !b.tagline.is_empty()
                                || !b.capabilities.is_empty()
                        }
                        Err(_) => false,
                    },
                    Err(_) => false,
                };
                (i, usable)
            });
        }
        let mut keep = vec![false; summaries.len()];
        while let Some(res) = set.join_next().await {
            if let Ok((i, usable)) = res {
                keep[i] = usable;
            }
        }
        let out = summaries
            .into_iter()
            .zip(keep)
            .filter_map(|(s, k)| k.then_some(s))
            .collect();
        Ok(out)
    }

    async fn pull_brain(
        &self,
        owner: &str,
        name: &str,
        version: &str,
    ) -> Result<AgentBrain, BrainHubError> {
        let url = format!("{}/brains/{}/{}/{}/pull", self.base, owner, name, version);
        let bytes = fetch_bytes(&self.http, &self.token, &url).await?;
        brain_from_blob(owner, name, version, bytes)
    }
}

/// GET a URL as JSON, with the optional Bearer token. Free fn so it can run in
/// spawned (concurrent) tasks that can't borrow `&self`.
async fn fetch_json(
    http: &reqwest::Client,
    token: &Option<String>,
    url: &str,
) -> Result<serde_json::Value, BrainHubError> {
    let mut req = http.get(url);
    if let Some(t) = token {
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

/// GET a URL as raw bytes (a .brain blob may be JSON or HDF5).
async fn fetch_bytes(
    http: &reqwest::Client,
    token: &Option<String>,
    url: &str,
) -> Result<Vec<u8>, BrainHubError> {
    let mut req = http.get(url);
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| BrainHubError::Http(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(BrainHubError::Http(format!("status {}", resp.status())));
    }
    Ok(resp
        .bytes()
        .await
        .map_err(|e| BrainHubError::Http(e.to_string()))?
        .to_vec())
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

/// Normalize a pulled `.brain` blob (JSON or clawhdf5 HDF5) into card fields.
/// `name` is the registry name (used for the resource URL).
fn brain_from_blob(
    owner: &str,
    name: &str,
    version: &str,
    bytes: Vec<u8>,
) -> Result<AgentBrain, BrainHubError> {
    match bytes.first() {
        // JSON .brain ("{ ...").
        Some(b'{') => {
            let json: serde_json::Value =
                serde_json::from_slice(&bytes).map_err(|e| BrainHubError::Parse(e.to_string()))?;
            Ok(parse_brain_json(owner, name, version, &json))
        }
        // HDF5 .brain (magic "\x89HDF\r\n\x1a\n").
        _ if bytes.starts_with(b"\x89HDF\r\n\x1a\n") => {
            parse_brain_hdf5(owner, name, version, bytes)
        }
        _ => Err(BrainHubError::Parse("unrecognized .brain format".into())),
    }
}

/// Normalize a JSON `.brain` into card fields.
fn parse_brain_json(
    owner: &str,
    name: &str,
    version: &str,
    json: &serde_json::Value,
) -> AgentBrain {
    let str_at = |p: &str| {
        json.pointer(p)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    };
    let display = json
        .pointer("/meta/brain_name")
        .and_then(|v| v.as_str())
        .unwrap_or(name);
    let tools = json
        .pointer("/skills/tool_defs")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let (agent_md, soul_md, skills_md) = (
        str_at("/identity/agent_md"),
        str_at("/identity/soul_md"),
        str_at("/skills/skills_md"),
    );
    build_agent_brain(BrainParts {
        owner,
        reg_name: name,
        display_name: display,
        version,
        agent_md: &agent_md,
        soul_md: &soul_md,
        skills_md: &skills_md,
        tools,
    })
}

/// Normalize a clawhdf5 HDF5 `.brain` into card fields (same datasets as JSON).
fn parse_brain_hdf5(
    owner: &str,
    name: &str,
    version: &str,
    bytes: Vec<u8>,
) -> Result<AgentBrain, BrainHubError> {
    let file = clawhdf5::File::from_bytes(bytes)
        .map_err(|e| BrainHubError::Parse(format!("hdf5: {e}")))?;
    let read = |path: &str| -> String {
        file.dataset(path)
            .ok()
            .and_then(|ds| ds.read_selection(&clawhdf5::Selection::All).ok())
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default()
    };
    let tools = serde_json::from_str::<Vec<serde_json::Value>>(&read("skills/tool_defs_json"))
        .ok()
        .map(|a| {
            a.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let (agent_md, soul_md, skills_md) = (
        read("identity/agent_md"),
        read("identity/soul_md"),
        read("skills/skills_md"),
    );
    Ok(build_agent_brain(BrainParts {
        owner,
        reg_name: name,
        display_name: name,
        version,
        agent_md: &agent_md,
        soul_md: &soul_md,
        skills_md: &skills_md,
        tools,
    }))
}

/// Inputs to `build_agent_brain` (grouped to keep the arg count sane).
struct BrainParts<'a> {
    owner: &'a str,
    /// Registry name (used for the shareable URL).
    reg_name: &'a str,
    /// Display name shown on the card (meta brain_name, falls back to reg_name).
    display_name: &'a str,
    version: &'a str,
    agent_md: &'a str,
    soul_md: &'a str,
    skills_md: &'a str,
    tools: Vec<String>,
}

/// Shared normalization from the brain's markdown fields → card fields.
fn build_agent_brain(p: BrainParts) -> AgentBrain {
    let mut skills = md_headings(p.skills_md);
    skills.truncate(8);
    let mut tools = p.tools;
    tools.truncate(8);

    let mut capabilities = md_bullets(p.agent_md);
    if capabilities.is_empty() {
        capabilities = md_headings(p.agent_md)
            .into_iter()
            .filter(|h| h != "Purpose")
            .collect();
    }
    capabilities.truncate(6);

    let tagline = first_paragraph(p.soul_md)
        .or_else(|| first_paragraph(p.agent_md))
        .unwrap_or_default();

    AgentBrain {
        owner: p.owner.to_string(),
        name: p.display_name.to_string(),
        version: p.version.to_string(),
        tagline,
        skills,
        tools,
        capabilities,
        resource_url: format!("{WEB_BASE}/{}/{}", p.owner, p.reg_name),
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
            resource_url: format!("{WEB_BASE}/{owner}/{name}"),
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
        let b = parse_brain_json("redclawsystems", "general-assistant", "1.0.0", &json);
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
