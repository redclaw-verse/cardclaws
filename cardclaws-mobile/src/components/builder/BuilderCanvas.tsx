// Builder v1 (PRD §6.1.5, Phase 1 weeks 5-6): full-screen face preview with a
// floating toolbar to add text/logo layers, set the background, and undo/redo.
// Backed by the command/undo cardStore.

import { useState } from "react";
import { Pressable, StyleSheet, Text, TextInput, useWindowDimensions, View } from "react-native";
import { CardFace } from "../card/CardFace";
import { ColorPaletteEditor } from "./ColorPaletteEditor";
import { LayerOrderPanel } from "./LayerOrderPanel";
import { LayerPropertySheet } from "./LayerPropertySheet";
import { ProfileEditor } from "./ProfileEditor";
import { Side, newLayerId, useCardStore } from "../../stores/cardStore";
import { ShapeLayer, TextLayer } from "../../types/card";

export function BuilderCanvas({ side = "face" as Side }: { side?: Side }) {
  const { width, height } = useWindowDimensions();
  const card = useCardStore((s) => s.card);
  const addLayer = useCardStore((s) => s.addLayer);
  const setBackground = useCardStore((s) => s.setBackground);
  const undo = useCardStore((s) => s.undo);
  const redo = useCardStore((s) => s.redo);
  const canUndo = useCardStore((s) => s.canUndo());
  const canRedo = useCardStore((s) => s.canRedo());

  const [draftText, setDraftText] = useState("");
  const [orderOpen, setOrderOpen] = useState(false);
  const [profileOpen, setProfileOpen] = useState(false);
  const [selectedLayerId, setSelectedLayerId] = useState<string | null>(null);

  if (!card) return null;
  const cardWidth = Math.min(width * 0.9, 360);
  const cardHeight = cardWidth * 1.5;

  const addText = () => {
    const text = draftText.trim() || "New text";
    const layer: TextLayer = {
      id: newLayerId(),
      type: "text",
      x: 0.1,
      y: 0.12 + card[side].layers.length * 0.08,
      width: 0.8,
      height: 0.12,
      opacity: 1,
      zIndex: card[side].layers.length + 1,
      text,
      fontFamily: "System",
      fontWeight: 700,
      fontSize: 26,
      lineHeight: 30,
      letterSpacing: 0,
      color: "#f5f5f7",
      align: "left",
    };
    addLayer(side, layer);
    setDraftText("");
  };

  const addShape = () => {
    const layer: ShapeLayer = {
      id: newLayerId(),
      type: "shape",
      x: 0.1,
      y: 0.5,
      width: 0.35,
      height: 0.2,
      opacity: 1,
      zIndex: card[side].layers.length + 1,
      shape: "rectangle",
      fill: "#ff3b30",
      strokeWidth: 0,
      cornerRadius: 12,
    };
    addLayer(side, layer);
  };

  return (
    <View style={styles.root}>
      <View style={styles.canvasArea}>
        <CardFace side={card[side]} width={cardWidth} height={cardHeight} />
      </View>

      <ColorPaletteEditor
        colors={card.palette.colors}
        selected={card[side].background.value}
        onSelect={(hex) => setBackground(side, { type: "solid", value: hex })}
      />

      <View style={styles.toolbar}>
        <TextInput
          style={styles.input}
          placeholder="Layer text…"
          placeholderTextColor="#6b6b70"
          value={draftText}
          onChangeText={setDraftText}
          onSubmitEditing={addText}
        />
        <ToolButton label="Add" onPress={addText} />
        <ToolButton label="↶" onPress={undo} disabled={!canUndo} />
        <ToolButton label="↷" onPress={redo} disabled={!canRedo} />
      </View>

      <View style={styles.secondaryBar}>
        <ToolButton label="Shape" onPress={addShape} />
        <ToolButton label="Layers" onPress={() => setOrderOpen(true)} />
        <ToolButton label="Profile" onPress={() => setProfileOpen(true)} />
      </View>

      <LayerOrderPanel
        side={side}
        visible={orderOpen}
        onClose={() => setOrderOpen(false)}
        onSelect={setSelectedLayerId}
      />
      <LayerPropertySheet side={side} layerId={selectedLayerId} onClose={() => setSelectedLayerId(null)} />
      <ProfileEditor visible={profileOpen} onClose={() => setProfileOpen(false)} />
    </View>
  );
}

function ToolButton({
  label,
  onPress,
  disabled,
}: {
  label: string;
  onPress: () => void;
  disabled?: boolean;
}) {
  return (
    <Pressable
      onPress={onPress}
      disabled={disabled}
      style={[styles.btn, disabled && styles.btnDisabled]}
    >
      <Text style={styles.btnText}>{label}</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  canvasArea: { flex: 1, alignItems: "center", justifyContent: "center" },
  toolbar: {
    flexDirection: "row",
    gap: 8,
    padding: 16,
    alignItems: "center",
  },
  secondaryBar: {
    flexDirection: "row",
    gap: 8,
    paddingHorizontal: 16,
    paddingBottom: 16,
  },
  input: {
    flex: 1,
    backgroundColor: "#1a1a1f",
    color: "#f5f5f7",
    borderRadius: 12,
    paddingHorizontal: 14,
    paddingVertical: 10,
  },
  btn: {
    backgroundColor: "#ff3b30",
    borderRadius: 12,
    paddingHorizontal: 16,
    paddingVertical: 10,
  },
  btnDisabled: { opacity: 0.35 },
  btnText: { color: "#fff", fontWeight: "600", fontSize: 16 },
});
