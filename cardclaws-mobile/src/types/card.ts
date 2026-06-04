// Canonical CardDefinition types — the single source of truth shared with the
// backend (mirrors cardclaws-types::card in Rust; PRD §8.3). The back side key
// is `back` (NOT the mangled `cardclaws` from the PRD §12.1 draft).

export type LayerType =
  | "background"
  | "text"
  | "logo"
  | "shape"
  | "qr"
  | "contact"
  | "video"
  | "particle"
  | "animatedGradient";

export type EntryAnimationType = "rise" | "fade" | "scale" | "deal" | "none";

export interface AnimationConfig {
  kind: string;
  durationMs: number;
  delayMs: number;
}

export interface BaseLayer {
  id: string;
  type: LayerType;
  /** 0.0–1.0 fractions of the canvas. */
  x: number;
  y: number;
  width: number;
  height: number;
  opacity: number;
  zIndex: number;
  entryAnimation?: AnimationConfig | null;
  loopAnimation?: AnimationConfig | null;
}

export interface TextLayer extends BaseLayer {
  type: "text";
  text: string;
  fontFamily: string;
  fontWeight: number;
  fontSize: number;
  lineHeight: number;
  letterSpacing: number;
  color: string;
  align: "left" | "center" | "right";
}

export interface LogoLayer extends BaseLayer {
  type: "logo";
  r2Key: string;
}

export interface ContactFields {
  phone?: string;
  email?: string;
  website?: string;
  linkedin?: string;
  company?: string;
  title?: string;
}

export interface ContactLayer extends BaseLayer {
  type: "contact";
  fields: ContactFields;
}

export type Layer = TextLayer | LogoLayer | ContactLayer | BaseLayer;

export interface GradientStop {
  color: string;
  position: number;
}

export interface BackgroundConfig {
  type: "solid" | "gradient" | "image";
  value?: string;
  r2Key?: string;
  stops?: GradientStop[];
  angle?: number;
}

export interface CardSide {
  layers: Layer[];
  background: BackgroundConfig;
  entryAnimation: EntryAnimationType;
}

export interface PaletteColor {
  name: string;
  hex: string;
  role?: "primary" | "secondary" | "accent" | "background" | "text" | null;
}

export interface ColorPalette {
  colors: PaletteColor[];
}

export interface CardSettings {
  flipGesture: "swipe" | "doubleTap" | "both";
  flipDurationMs: number;
  ambientModeEnabled: boolean;
  ambientModeDelayMs: number;
  hapticEnabled: boolean;
}

export interface CardDefinition {
  id: string;
  ownerId: string;
  handle: string;
  version: number;
  face: CardSide;
  back: CardSide;
  palette: ColorPalette;
  settings: CardSettings;
}

export const DEFAULT_SETTINGS: CardSettings = {
  flipGesture: "both",
  flipDurationMs: 400,
  ambientModeEnabled: true,
  ambientModeDelayMs: 5000,
  hapticEnabled: true,
};

export function emptySide(backgroundHex = "#101014"): CardSide {
  return {
    layers: [],
    background: { type: "solid", value: backgroundHex },
    entryAnimation: "fade",
  };
}
