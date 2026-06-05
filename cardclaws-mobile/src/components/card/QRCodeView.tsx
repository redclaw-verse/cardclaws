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
  // Guard empty/invalid input: the qrcode library throws "No input text" on an
  // empty string, which would crash the whole render. Render a blank box instead.
  const matrix = useMemo(() => {
    try {
      return value && value.trim() ? qrMatrix(value) : [];
    } catch {
      return [];
    }
  }, [value]);
  if (matrix.length === 0) {
    return <View style={[styles.root, { width: size, height: size, backgroundColor: background }]} />;
  }
  const modules = matrix.length + quietZone * 2;
  // Integer cell size so modules tile exactly — fractional cells leave
  // sub-pixel seams that show as white lines through the code. Center the
  // (slightly smaller) grid inside the requested size.
  const cell = Math.max(1, Math.floor(size / modules));
  const rendered = cell * modules;
  const pad = Math.round((size - rendered) / 2);

  return (
    <View style={[styles.root, { width: size, height: size, backgroundColor: background }]}>
      {matrix.map((row, r) =>
        row.map((dark, c) =>
          dark ? (
            <View
              key={`${r}-${c}`}
              style={{
                position: "absolute",
                left: pad + (c + quietZone) * cell,
                top: pad + (r + quietZone) * cell,
                width: cell,
                height: cell,
                backgroundColor: color,
              }}
            />
          ) : null,
        ),
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  root: { borderRadius: 8, overflow: "hidden" },
});
