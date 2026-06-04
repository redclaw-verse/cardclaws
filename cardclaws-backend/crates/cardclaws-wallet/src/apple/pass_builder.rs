//! Constructs `pass.json` per the PassKit spec (PRD §6.3.1). Type `generic`.

use serde_json::{json, Value};

use super::PassInput;

pub fn build_pass_json(input: &PassInput) -> Value {
    let mut secondary = Vec::new();
    if let Some(company) = &input.company {
        secondary.push(json!({ "key": "company", "label": "COMPANY", "value": company }));
    }
    if let Some(email) = &input.email {
        secondary.push(json!({ "key": "email", "label": "EMAIL", "value": email }));
    }

    let mut auxiliary = Vec::new();
    if let Some(phone) = &input.phone {
        auxiliary.push(json!({ "key": "phone", "label": "PHONE", "value": phone }));
    }
    if let Some(website) = &input.website {
        auxiliary.push(json!({ "key": "website", "label": "WEBSITE", "value": website }));
    }

    let mut back = vec![json!({
        "key": "profile",
        "label": "PROFILE",
        "value": input.profile_url,
    })];
    if let Some(email) = &input.email {
        back.push(json!({ "key": "back_email", "label": "EMAIL", "value": email }));
    }

    json!({
        "formatVersion": 1,
        "passTypeIdentifier": input.pass_type_id,
        "teamIdentifier": input.team_id,
        "organizationName": input.organization_name,
        "serialNumber": input.serial_number,
        "description": format!("{}'s CardClaws card", input.holder_name),
        "barcode": {
            "format": "PKBarcodeFormatQR",
            "message": input.profile_url,
            "messageEncoding": "iso-8859-1",
        },
        "barcodes": [{
            "format": "PKBarcodeFormatQR",
            "message": input.profile_url,
            "messageEncoding": "iso-8859-1",
        }],
        "generic": {
            "headerFields": [
                { "key": "name", "value": input.holder_name }
            ],
            "primaryFields": [
                { "key": "title", "value": input.title.clone().unwrap_or_default() }
            ],
            "secondaryFields": secondary,
            "auxiliaryFields": auxiliary,
            "backFields": back,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> PassInput {
        PassInput {
            serial_number: "serial-123".into(),
            pass_type_id: "pass.com.cardclaws.card".into(),
            team_id: "TEAM123".into(),
            organization_name: "CardClaws".into(),
            holder_name: "Omar Sobh".into(),
            title: Some("Founder".into()),
            company: Some("RedClaw".into()),
            email: Some("omar@redclaw.dev".into()),
            phone: Some("+15551234567".into()),
            website: Some("https://redclaw.dev".into()),
            profile_url: "https://cardclaws.com/omar".into(),
            background_hex: "#101014".into(),
        }
    }

    #[test]
    fn pass_json_has_required_passkit_fields() {
        let pass = build_pass_json(&input());
        for key in [
            "formatVersion",
            "passTypeIdentifier",
            "teamIdentifier",
            "organizationName",
            "serialNumber",
            "description",
            "barcode",
            "generic",
        ] {
            assert!(pass.get(key).is_some(), "missing required field {key}");
        }
        assert_eq!(pass["formatVersion"], 1);
    }

    #[test]
    fn barcode_message_is_profile_url() {
        let pass = build_pass_json(&input());
        assert_eq!(pass["barcode"]["message"], "https://cardclaws.com/omar");
        assert_eq!(pass["barcode"]["format"], "PKBarcodeFormatQR");
    }
}
