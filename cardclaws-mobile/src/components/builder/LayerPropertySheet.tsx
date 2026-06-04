// Property editor for a selected layer (PRD §6.1.5). Fields vary by layer type;
// edits route through cardStore.updateLayer so they're undoable.

import { Modal, Pressable, StyleSheet, Text, TextInput, View } from "react-native";
import { Side, useCardStore } from "../../stores/cardStore";
import { ShapeLayer, TextLayer } from "../../types/card";
import { ColorPaletteEditor } from "./ColorPaletteEditor";

interface Props {
  side: Side;
  layerId: string | null;
  onClose: () => void;
}

export function LayerPropertySheet({ side, layerId, onClose }: Props) {
  const card = useCardStore((s) => s.card);
  const updateLayer = useCardStore((s) => s.updateLayer);
  const removeLayer = useCardStore((s) => s.removeLayer);

  const layer = card?.[side].layers.find((l) => l.id === layerId) ?? null;
  const palette = card?.palette.colors ?? [];

  const stepOpacity = (delta: number) => {
    if (!layer) return;
    const opacity = Math.max(0, Math.min(1, Math.round((layer.opacity + delta) * 10) / 10));
    updateLayer(side, layer.id, { opacity });
  };

  return (
    <Modal visible={!!layer} transparent animationType="slide" onRequestClose={onClose}>
      <Pressable style={styles.backdrop} onPress={onClose}>
        <Pressable style={styles.sheet} onPress={(e) => e.stopPropagation()}>
          {layer && (
            <>
              <Text style={styles.title}>{layer.type} layer</Text>

              {layer.type === "text" && (
                <>
                  <Field
                    label="Text"
                    value={(layer as TextLayer).text}
                    onChangeText={(text) => updateLayer(side, layer.id, { text } as Partial<TextLayer>)}
                  />
                  <Text style={styles.label}>Color</Text>
                  <ColorPaletteEditor
                    colors={palette}
                    selected={(layer as TextLayer).color}
                    onSelect={(color) => updateLayer(side, layer.id, { color } as Partial<TextLayer>)}
                  />
                </>
              )}

              {layer.type === "shape" && (
                <>
                  <Text style={styles.label}>Fill</Text>
                  <ColorPaletteEditor
                    colors={palette}
                    selected={(layer as ShapeLayer).fill}
                    onSelect={(fill) => updateLayer(side, layer.id, { fill } as Partial<ShapeLayer>)}
                  />
                </>
              )}

              <View style={styles.stepRow}>
                <Text style={styles.label}>Opacity {Math.round(layer.opacity * 100)}%</Text>
                <View style={styles.steppers}>
                  <Stepper label="−" onPress={() => stepOpacity(-0.1)} />
                  <Stepper label="+" onPress={() => stepOpacity(0.1)} />
                </View>
              </View>

              <Pressable
                style={styles.delete}
                onPress={() => {
                  removeLayer(side, layer.id);
                  onClose();
                }}
              >
                <Text style={styles.deleteText}>Delete layer</Text>
              </Pressable>
            </>
          )}
        </Pressable>
      </Pressable>
    </Modal>
  );
}

function Field({
  label,
  value,
  onChangeText,
}: {
  label: string;
  value: string;
  onChangeText: (t: string) => void;
}) {
  return (
    <>
      <Text style={styles.label}>{label}</Text>
      <TextInput style={styles.input} value={value} onChangeText={onChangeText} />
    </>
  );
}

function Stepper({ label, onPress }: { label: string; onPress: () => void }) {
  return (
    <Pressable style={styles.stepper} onPress={onPress}>
      <Text style={styles.stepperText}>{label}</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  backdrop: { flex: 1, backgroundColor: "rgba(0,0,0,0.5)", justifyContent: "flex-end" },
  sheet: {
    backgroundColor: "#15151a",
    borderTopLeftRadius: 24,
    borderTopRightRadius: 24,
    padding: 20,
    paddingBottom: 36,
    gap: 12,
  },
  title: { color: "#f5f5f7", fontSize: 18, fontWeight: "700", textTransform: "capitalize" },
  label: { color: "#9a9aa0", fontSize: 14 },
  input: {
    backgroundColor: "#222228",
    color: "#f5f5f7",
    borderRadius: 12,
    paddingHorizontal: 14,
    paddingVertical: 12,
  },
  stepRow: { flexDirection: "row", alignItems: "center", justifyContent: "space-between" },
  steppers: { flexDirection: "row", gap: 8 },
  stepper: {
    backgroundColor: "#222228",
    width: 44,
    height: 44,
    borderRadius: 12,
    alignItems: "center",
    justifyContent: "center",
  },
  stepperText: { color: "#f5f5f7", fontSize: 22, fontWeight: "700" },
  delete: { paddingVertical: 14, alignItems: "center" },
  deleteText: { color: "#ff453a", fontWeight: "600", fontSize: 16 },
});
