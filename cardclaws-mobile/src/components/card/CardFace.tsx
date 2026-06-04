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
  TextLayer,
} from "../../types/card";
import { API_BASE } from "../../api/client";

interface Props {
  side: CardSide;
  width: number;
  height: number;
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

function LayerView({ layer, width, height }: { layer: Layer; width: number; height: number }) {
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
    default:
      return <View style={frame} />;
  }
}

export function CardFace({ side, width, height }: Props) {
  const ordered = [...side.layers].sort((a, b) => a.zIndex - b.zIndex);
  return (
    <View style={[styles.face, { width, height }, backgroundStyle(side.background)]}>
      {ordered.map((layer) => (
        <LayerView key={layer.id} layer={layer} width={width} height={height} />
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  face: {
    borderRadius: 24,
    overflow: "hidden",
  },
  contactLine: {
    color: "#f5f5f7",
    fontSize: 15,
    marginBottom: 6,
  },
});
