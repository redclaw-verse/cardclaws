//! vCard 3.0 (RFC 6350) generation from a card's contact data (PRD §17.3).
//!
//! The card `definition` is stored as opaque JSON, so we extract contact fields
//! from the first `contact`-type layer found on either side, falling back to the
//! account display name for the formatted name.

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactInfo {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub linkedin: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
}

/// Pull contact fields out of a card definition by scanning face/back layers for
/// the first `{"type":"contact","fields":{...}}` layer.
pub fn extract_contact(definition: &serde_json::Value) -> ContactInfo {
    for side in ["face", "back"] {
        if let Some(layers) = definition
            .get(side)
            .and_then(|s| s.get("layers"))
            .and_then(|l| l.as_array())
        {
            for layer in layers {
                if layer.get("type").and_then(|t| t.as_str()) == Some("contact") {
                    if let Some(fields) = layer.get("fields") {
                        if let Ok(info) = serde_json::from_value::<ContactInfo>(fields.clone()) {
                            return info;
                        }
                    }
                }
            }
        }
    }
    ContactInfo::default()
}

/// Build an RFC 6350 vCard 3.0 string. `display_name` becomes FN; the structured
/// N field is a best-effort split on the first space.
pub fn build_vcard(display_name: &str, contact: &ContactInfo) -> String {
    let (first, last) = split_name(display_name);
    let mut lines = vec![
        "BEGIN:VCARD".to_string(),
        "VERSION:3.0".to_string(),
        format!("FN:{}", escape(display_name)),
        format!("N:{};{};;;", escape(&last), escape(&first)),
    ];
    if let Some(org) = &contact.company {
        lines.push(format!("ORG:{}", escape(org)));
    }
    if let Some(title) = &contact.title {
        lines.push(format!("TITLE:{}", escape(title)));
    }
    if let Some(phone) = &contact.phone {
        lines.push(format!("TEL;TYPE=CELL:{}", escape(phone)));
    }
    if let Some(email) = &contact.email {
        lines.push(format!("EMAIL:{}", escape(email)));
    }
    if let Some(url) = &contact.website {
        lines.push(format!("URL:{}", escape(url)));
    }
    if let Some(linkedin) = &contact.linkedin {
        lines.push(format!(
            "X-SOCIALPROFILE;TYPE=linkedin:{}",
            escape(linkedin)
        ));
    }
    lines.push("END:VCARD".to_string());
    // vCard uses CRLF line endings.
    lines.join("\r\n") + "\r\n"
}

fn split_name(display_name: &str) -> (String, String) {
    match display_name.split_once(' ') {
        Some((first, last)) => (first.to_string(), last.to_string()),
        None => (display_name.to_string(), String::new()),
    }
}

/// Escape the characters that are special in vCard property values.
fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extracts_contact_from_back_layer() {
        let def = json!({
            "face": { "layers": [] },
            "back": { "layers": [
                { "type": "text", "text": "Omar" },
                { "type": "contact", "fields": {
                    "phone": "+15551234567",
                    "email": "omar@redclaw.dev",
                    "company": "RedClaw",
                    "title": "Founder"
                }}
            ]}
        });
        let c = extract_contact(&def);
        assert_eq!(c.email.as_deref(), Some("omar@redclaw.dev"));
        assert_eq!(c.company.as_deref(), Some("RedClaw"));
    }

    #[test]
    fn builds_valid_vcard_structure() {
        let contact = ContactInfo {
            phone: Some("+15551234567".into()),
            email: Some("omar@redclaw.dev".into()),
            company: Some("RedClaw".into()),
            title: Some("Founder".into()),
            website: Some("https://redclaw.dev".into()),
            linkedin: None,
        };
        let vcf = build_vcard("Omar Sobh", &contact);
        assert!(vcf.starts_with("BEGIN:VCARD\r\nVERSION:3.0\r\n"));
        assert!(vcf.contains("FN:Omar Sobh\r\n"));
        assert!(vcf.contains("N:Sobh;Omar;;;\r\n"));
        assert!(vcf.contains("ORG:RedClaw\r\n"));
        assert!(vcf.contains("TITLE:Founder\r\n"));
        assert!(vcf.contains("TEL;TYPE=CELL:+15551234567\r\n"));
        assert!(vcf.contains("EMAIL:omar@redclaw.dev\r\n"));
        assert!(vcf.trim_end().ends_with("END:VCARD"));
    }

    #[test]
    fn escapes_special_characters() {
        let contact = ContactInfo {
            company: Some("Red;Claw, Inc".into()),
            ..Default::default()
        };
        let vcf = build_vcard("Solo", &contact);
        assert!(vcf.contains("ORG:Red\\;Claw\\, Inc"));
    }
}
