// Renders a QR code as a grid of cells from the on-device qrMatrix (PRD §16,
// QR layer). Pure RN views — no native QR dependency.

import { useMemo } from "react";
import { StyleSheet, View } from "react-native";
import { qrMatrix } from "../../engine/renderer/qrGenerator";

interface Props {
  value: string;
  size: number;
  /** Quiet-zone border in cells (QR spec recommends 4). */
  quietZone?: number;
  color?: string;
  background?: string;
}

export function QRCodeView({
  value,
  size,
  quietZone = 2,
  color = "#000000",
  background = "#ffffff",
}: Props) {
  const matrix = useMemo(() => qrMatrix(value), [value]);
  const modules = matrix.length + quietZone * 2;
  const cell = size / modules;

  return (
    <View style={[styles.root, { width: size, height: size, backgroundColor: background }]}>
      {matrix.map((row, r) => (
        <View key={r} style={styles.row}>
          {row.map((dark, c) => (
            <View
              key={c}
              style={{
                position: "absolute",
                left: (c + quietZone) * cell,
                top: (r + quietZone) * cell,
                width: cell,
                height: cell,
                backgroundColor: dark ? color : "transparent",
              }}
            />
          ))}
        </View>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  root: { borderRadius: 8, overflow: "hidden" },
  row: { ...StyleSheet.absoluteFillObject },
});
