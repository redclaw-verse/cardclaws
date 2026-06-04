// Palette swatch row. Tapping a swatch sets the current side's background
// (PRD §6.1.3). Palette colours are extracted on-device via colorQuantizer or
// entered manually.

import { Pressable, ScrollView, StyleSheet, View } from "react-native";
import { PaletteColor } from "../../types/card";

interface Props {
  colors: PaletteColor[];
  selected?: string;
  onSelect: (hex: string) => void;
}

const FALLBACK: PaletteColor[] = [
  { name: "Ink", hex: "#101014" },
  { name: "Snow", hex: "#f5f5f7" },
  { name: "Claw", hex: "#ff3b30" },
  { name: "Ocean", hex: "#1b1b2f" },
  { name: "Moss", hex: "#1f3d2b" },
];

export function ColorPaletteEditor({ colors, selected, onSelect }: Props) {
  const swatches = colors.length > 0 ? colors : FALLBACK;
  return (
    <ScrollView horizontal showsHorizontalScrollIndicator={false} contentContainerStyle={styles.row}>
      {swatches.map((c) => (
        <Pressable
          key={c.hex}
          accessibilityLabel={`Background ${c.name}`}
          onPress={() => onSelect(c.hex)}
          style={[
            styles.swatch,
            { backgroundColor: c.hex },
            selected === c.hex && styles.selected,
          ]}
        >
          <View />
        </Pressable>
      ))}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  row: { gap: 12, paddingHorizontal: 16, paddingVertical: 12 },
  swatch: {
    width: 44,
    height: 44,
    borderRadius: 22,
    borderWidth: 2,
    borderColor: "rgba(255,255,255,0.15)",
  },
  selected: { borderColor: "#ffffff", borderWidth: 3 },
});
