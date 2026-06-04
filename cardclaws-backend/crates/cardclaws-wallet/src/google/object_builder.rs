//! Builds the `savetowallet` JWT claims, including the inline GenericObject.

use serde_json::{json, Value};

use super::GoogleInput;

/// Build the JWT claims for the save link. The GenericObject is embedded under
/// `payload.genericObjects` so the link is self-contained.
pub fn build_claims(input: &GoogleInput) -> Value {
    json!({
        "iss": input.issuer_email,
        "aud": "google",
        "typ": "savetowallet",
        "origins": [],
        "payload": {
            "genericObjects": [ build_object(input) ]
        }
    })
}

fn build_object(input: &GoogleInput) -> Value {
    let subheader = match (&input.title, &input.company) {
        (Some(t), Some(c)) => format!("{t} · {c}"),
        (Some(t), None) => t.clone(),
        (None, Some(c)) => c.clone(),
        (None, None) => String::new(),
    };

    json!({
        "id": input.object_id,
        "classId": input.class_id,
        "state": "ACTIVE",
        "hexBackgroundColor": input.background_hex,
        "cardTitle": {
            "defaultValue": { "language": "en", "value": "CardClaws" }
        },
        "header": {
            "defaultValue": { "language": "en", "value": input.holder_name }
        },
        "subheader": {
            "defaultValue": { "language": "en", "value": subheader }
        },
        "barcode": {
            "type": "QR_CODE",
            "value": input.profile_url,
            "alternateText": input.profile_url
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> GoogleInput {
        GoogleInput {
            issuer_email: "sa@project.iam.gserviceaccount.com".into(),
            class_id: "338800.cardclaws_generic".into(),
            object_id: "338800.card-abc".into(),
            holder_name: "Omar Sobh".into(),
            title: Some("Founder".into()),
            company: Some("RedClaw".into()),
            profile_url: "https://cardclaws.com/omar".into(),
            background_hex: "#101014".into(),
        }
    }

    #[test]
    fn claims_have_savetowallet_shape() {
        let c = build_claims(&input());
        assert_eq!(c["aud"], "google");
        assert_eq!(c["typ"], "savetowallet");
        assert_eq!(c["iss"], "sa@project.iam.gserviceaccount.com");
        let obj = &c["payload"]["genericObjects"][0];
        assert_eq!(obj["id"], "338800.card-abc");
        assert_eq!(obj["classId"], "338800.cardclaws_generic");
    }

    #[test]
    fn barcode_is_qr_to_profile() {
        let c = build_claims(&input());
        let barcode = &c["payload"]["genericObjects"][0]["barcode"];
        assert_eq!(barcode["type"], "QR_CODE");
        assert_eq!(barcode["value"], "https://cardclaws.com/omar");
    }

    #[test]
    fn header_is_holder_and_subheader_combines_title_company() {
        let c = build_claims(&input());
        let obj = &c["payload"]["genericObjects"][0];
        assert_eq!(obj["header"]["defaultValue"]["value"], "Omar Sobh");
        assert_eq!(
            obj["subheader"]["defaultValue"]["value"],
            "Founder · RedClaw"
        );
    }
}
