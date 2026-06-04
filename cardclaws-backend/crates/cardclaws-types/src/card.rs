//! The canonical `CardDefinition` — the single source of truth for a card,
//! shared by the mobile renderer, the web profile, and the backend. Mirrors the
//! TypeScript definition in PRD §8.3.
//!
//! NOTE: the back side key is `back` (NOT `cardclaws` as printed in the mangled
//! PRD §12.1 JSON schema — see the plan's review section A2). It round-trips to
//! the `cards.definition` JSONB column.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardDefinition {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub handle: String,
    pub version: i32,
    pub face: CardSide,
    pub back: CardSide,
    pub palette: ColorPalette,
    pub settings: CardSettings,
    pub profile: ProfileData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CardSide {
    pub layers: Vec<Layer>,
    pub background: BackgroundConfig,
    pub entry_animation: EntryAnimationType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum EntryAnimationType {
    Rise,
    Fade,
    Scale,
    Deal,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum BackgroundConfig {
    Solid { value: String },
    Gradient { value: GradientConfig },
    Image { r2_key: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GradientConfig {
    /// linear | radial | conic
    pub kind: String,
    pub stops: Vec<GradientStop>,
    /// Angle in degrees for linear gradients.
    pub angle: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GradientStop {
    pub color: String,
    /// 0.0 - 1.0
    pub position: f32,
}

/// All layer variants. Geometry fields (x/y/width/height) are fractions of the
/// canvas (0.0 - 1.0) so a card renders identically at any device size.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Layer {
    Background(BaseLayer),
    Text(TextLayer),
    Logo(LogoLayer),
    Shape(ShapeLayer),
    Qr(BaseLayer),
    Contact(ContactLayer),
    Video(LogoLayer),
    Particle(BaseLayer),
    AnimatedGradient(BaseLayer),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaseLayer {
    pub id: Uuid,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub opacity: f32,
    pub z_index: i32,
    #[serde(default)]
    pub entry_animation: Option<AnimationConfig>,
    #[serde(default)]
    pub loop_animation: Option<AnimationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TextLayer {
    #[serde(flatten)]
    pub base: BaseLayer,
    pub text: String,
    pub font_family: String,
    pub font_weight: i32,
    pub font_size: f32,
    pub line_height: f32,
    pub letter_spacing: f32,
    pub color: String,
    /// left | center | right
    pub align: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogoLayer {
    #[serde(flatten)]
    pub base: BaseLayer,
    pub r2_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShapeLayer {
    #[serde(flatten)]
    pub base: BaseLayer,
    /// rectangle | circle | line
    pub shape: String,
    pub fill: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: f32,
    pub corner_radius: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContactLayer {
    #[serde(flatten)]
    pub base: BaseLayer,
    pub fields: ContactFields,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContactFields {
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub linkedin: Option<String>,
    pub company: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AnimationConfig {
    /// fade | slide | scale | pulse | float | shimmer | none
    pub kind: String,
    pub duration_ms: u32,
    pub delay_ms: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ColorPalette {
    pub colors: Vec<PaletteColor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PaletteColor {
    pub name: String,
    pub hex: String,
    /// primary | secondary | accent | background | text | null
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CardSettings {
    /// swipe | doubleTap | both
    pub flip_gesture: String,
    pub flip_duration_ms: u32,
    pub ambient_mode_enabled: bool,
    pub ambient_mode_delay_ms: u32,
    pub haptic_enabled: bool,
}

impl Default for CardSettings {
    fn default() -> Self {
        Self {
            flip_gesture: "both".to_string(),
            flip_duration_ms: 400,
            ambient_mode_enabled: true,
            ambient_mode_delay_ms: 5000,
            haptic_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileData {
    #[serde(default)]
    pub bio: String,
    pub avatar_r2_key: Option<String>,
    #[serde(default)]
    pub links: Vec<ProfileLink>,
    #[serde(default)]
    pub portfolio: Vec<PortfolioItem>,
    #[serde(default)]
    pub testimonials: Vec<Testimonial>,
    #[serde(default)]
    pub contact_form_enabled: bool,
    #[serde(default)]
    pub theme_overrides: ThemeOverrides,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProfileLink {
    pub id: Uuid,
    #[serde(rename = "type")]
    pub link_type: String,
    pub label: String,
    pub url: String,
    pub icon_slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioItem {
    pub id: Uuid,
    /// image | video
    #[serde(rename = "type")]
    pub media_type: String,
    pub r2_key: String,
    pub caption: String,
    pub link_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Testimonial {
    pub id: Uuid,
    pub text: String,
    pub author_name: String,
    pub author_company: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ThemeOverrides {
    pub background_color: Option<String>,
    pub accent_color: Option<String>,
    pub text_color: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The back side must serialize under the key `back`, not `cardclaws`.
    #[test]
    fn card_definition_uses_back_key() {
        let card = CardDefinition {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            handle: "omar".to_string(),
            version: 1,
            face: empty_side(),
            back: empty_side(),
            palette: ColorPalette::default(),
            settings: CardSettings::default(),
            profile: ProfileData::default(),
        };
        let json = serde_json::to_value(&card).unwrap();
        assert!(json.get("back").is_some(), "back key must exist");
        assert!(
            json.get("cardclaws").is_none(),
            "the mangled `cardclaws` key must NOT exist"
        );
    }

    #[test]
    fn card_definition_round_trips() {
        let card = CardDefinition {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            handle: "omar".to_string(),
            version: 3,
            face: empty_side(),
            back: empty_side(),
            palette: ColorPalette::default(),
            settings: CardSettings::default(),
            profile: ProfileData::default(),
        };
        let json = serde_json::to_string(&card).unwrap();
        let back: CardDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(card, back);
    }

    fn empty_side() -> CardSide {
        CardSide {
            layers: vec![],
            background: BackgroundConfig::Solid {
                value: "#000000".to_string(),
            },
            entry_animation: EntryAnimationType::Fade,
        }
    }
}
