// Layer list with reorder + select (PRD §6.1.5). Layers shown top-to-bottom by
// descending z-index (front to back). Tapping a row selects it for editing.

import { Modal, Pressable, ScrollView, StyleSheet, Text, View } from "react-native";
import { Side, useCardStore } from "../../stores/cardStore";
import { Layer, TextLayer } from "../../types/card";

interface Props {
  side: Side;
  visible: boolean;
  onClose: () => void;
  onSelect: (layerId: string) => void;
}

function describe(layer: Layer): string {
  if (layer.type === "text") return `Text — “${(layer as TextLayer).text}”`;
  return layer.type.charAt(0).toUpperCase() + layer.type.slice(1);
}

export function LayerOrderPanel({ side, visible, onClose, onSelect }: Props) {
  const card = useCardStore((s) => s.card);
  const reorderLayer = useCardStore((s) => s.reorderLayer);

  const layers = [...(card?.[side].layers ?? [])].sort((a, b) => b.zIndex - a.zIndex);

  return (
    <Modal visible={visible} transparent animationType="slide" onRequestClose={onClose}>
      <Pressable style={styles.backdrop} onPress={onClose}>
        <Pressable style={styles.sheet} onPress={(e) => e.stopPropagation()}>
          <View style={styles.grabber} />
          <Text style={styles.title}>Layers</Text>
          <ScrollView>
            {layers.length === 0 && <Text style={styles.empty}>No layers yet.</Text>}
            {layers.map((layer) => (
              <View key={layer.id} style={styles.row}>
                <Pressable
                  style={styles.rowLabel}
                  onPress={() => {
                    onSelect(layer.id);
                    onClose();
                  }}
                >
                  <Text style={styles.rowText} numberOfLines={1}>
                    {describe(layer)}
                  </Text>
                </Pressable>
                <Pressable onPress={() => reorderLayer(side, layer.id, "up")} hitSlop={8}>
                  <Text style={styles.move}>↑</Text>
                </Pressable>
                <Pressable onPress={() => reorderLayer(side, layer.id, "down")} hitSlop={8}>
                  <Text style={styles.move}>↓</Text>
                </Pressable>
              </View>
            ))}
          </ScrollView>
        </Pressable>
      </Pressable>
    </Modal>
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
    maxHeight: "70%",
  },
  grabber: {
    alignSelf: "center",
    width: 40,
    height: 4,
    borderRadius: 2,
    backgroundColor: "#3a3a40",
    marginBottom: 8,
  },
  title: { color: "#f5f5f7", fontSize: 20, fontWeight: "700", marginBottom: 8 },
  empty: { color: "#6b6b70", paddingVertical: 12 },
  row: {
    flexDirection: "row",
    alignItems: "center",
    gap: 16,
    paddingVertical: 12,
    borderBottomColor: "#1a1a1f",
    borderBottomWidth: 1,
  },
  rowLabel: { flex: 1 },
  rowText: { color: "#f5f5f7", fontSize: 15 },
  move: { color: "#9a9aa0", fontSize: 22, width: 24, textAlign: "center" },
});
