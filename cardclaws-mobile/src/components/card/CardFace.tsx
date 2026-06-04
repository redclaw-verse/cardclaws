// Renders one CardSide at a given pixel size by absolutely positioning each
// layer using its fractional geometry. Background / text / logo / contact layers
// are supported in Phase 1 (Skia is reserved for the Pro shader/particle layers).

import { Image, StyleSheet, Text, View } from "react-native";
import {
  BackgroundConfig,
  CardSide,
  ContactLayer,
  Layer,
  LogoLayer,
  ShapeLayer,
  TextLayer,
} from "../../types/card";
import { API_BASE } from "../../api/client";
import { QRCodeView } from "./QRCodeView";

interface Props {
  side: CardSide;
  width: number;
  height: number;
  /** Corner radius; 0 for full-bleed full-screen cards. */
  borderRadius?: number;
  /** URL encoded by any `qr` layer on this side (the card's profile URL). */
  profileUrl?: string;
}

function backgroundStyle(bg: BackgroundConfig): { backgroundColor: string } {
  if (bg.type === "solid" && bg.value) return { backgroundColor: bg.value };
  if (bg.type === "gradient" && bg.stops?.length) {
    // Approximate a gradient's base with its first stop (a real gradient uses
    // a Skia/expo-linear-gradient fill in the Pro renderer).
    return { backgroundColor: bg.stops[0].color };
  }
  return { backgroundColor: "#101014" };
}

function assetUri(r2Key: string): string {
  return `${API_BASE}/assets/${r2Key}`;
}

/// Resolve an image background to a displayable URI. `value` carries a direct
/// URI (local file:// or content:// for the demo, or a full URL); otherwise the
/// r2Key resolves to the asset endpoint.
function backgroundImageUri(bg: BackgroundConfig): string | null {
  if (bg.type !== "image") return null;
  if (bg.value) return bg.value;
  if (bg.r2Key) return assetUri(bg.r2Key);
  return null;
}

function LayerView({
  layer,
  width,
  height,
  profileUrl,
}: {
  layer: Layer;
  width: number;
  height: number;
  profileUrl?: string;
}) {
  const frame = {
    position: "absolute" as const,
    left: layer.x * width,
    top: layer.y * height,
    width: layer.width * width,
    height: layer.height * height,
    opacity: layer.opacity,
  };

  switch (layer.type) {
    case "text": {
      const t = layer as TextLayer;
      return (
        <Text
          style={[
            frame,
            {
              color: t.color,
              fontSize: t.fontSize,
              fontWeight: String(t.fontWeight) as never,
              lineHeight: t.lineHeight,
              letterSpacing: t.letterSpacing,
              textAlign: t.align,
            },
          ]}
        >
          {t.text}
        </Text>
      );
    }
    case "logo": {
      const l = layer as LogoLayer;
      return (
        <Image
          style={frame}
          resizeMode="contain"
          source={{ uri: assetUri(l.r2Key) }}
        />
      );
    }
    case "contact": {
      const c = layer as ContactLayer;
      const lines = [c.fields.title, c.fields.company, c.fields.email, c.fields.phone].filter(
        Boolean,
      );
      return (
        <View style={frame}>
          {lines.map((line, i) => (
            <Text key={i} style={styles.contactLine}>
              {line}
            </Text>
          ))}
        </View>
      );
    }
    case "shape": {
      const s = layer as ShapeLayer;
      const radius =
        s.shape === "circle"
          ? Math.min(frame.width, frame.height)
          : s.shape === "line"
            ? 0
            : s.cornerRadius;
      return (
        <View
          style={[
            frame,
            {
              backgroundColor: s.fill,
              borderColor: s.stroke,
              borderWidth: s.stroke ? s.strokeWidth : 0,
              borderRadius: radius,
              height: s.shape === "line" ? Math.max(s.strokeWidth, 1) : frame.height,
            },
          ]}
        />
      );
    }
    case "qr":
      return (
        <View style={frame}>
          <QRCodeView value={profileUrl ?? ""} size={Math.min(frame.width, frame.height)} />
        </View>
      );
    default:
      return <View style={frame} />;
  }
}

export function CardFace({ side, width, height, borderRadius = 24, profileUrl }: Props) {
  const ordered = [...side.layers].sort((a, b) => a.zIndex - b.zIndex);
  const bgImage = backgroundImageUri(side.background);
  return (
    <View style={[styles.face, { width, height, borderRadius }, backgroundStyle(side.background)]}>
      {bgImage && (
        <Image source={{ uri: bgImage }} style={StyleSheet.absoluteFill} resizeMode="cover" />
      )}
      {ordered.map((layer) => (
        <LayerView
          key={layer.id}
          layer={layer}
          width={width}
          height={height}
          profileUrl={profileUrl}
        />
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  face: {
    overflow: "hidden",
  },
  contactLine: {
    color: "#f5f5f7",
    fontSize: 15,
    marginBottom: 6,
  },
});
