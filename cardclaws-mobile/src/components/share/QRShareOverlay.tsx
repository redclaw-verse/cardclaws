// Full-screen QR overlay shown for in-person exchange (PRD §6.5.1 QR modality).

import { Pressable, StyleSheet, Text, View } from "react-native";
import { QRCodeView } from "../card/QRCodeView";

interface Props {
  url: string;
  handle: string;
  onClose: () => void;
}

export function QRShareOverlay({ url, handle, onClose }: Props) {
  return (
    <Pressable style={styles.root} onPress={onClose}>
      <View style={styles.card}>
        <QRCodeView value={url} size={260} />
      </View>
      <Text style={styles.handle}>@{handle}</Text>
      <Text style={styles.hint}>Point a camera here to open the card</Text>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  root: {
    ...StyleSheet.absoluteFillObject,
    backgroundColor: "rgba(0,0,0,0.92)",
    alignItems: "center",
    justifyContent: "center",
    gap: 16,
  },
  card: { backgroundColor: "#fff", padding: 20, borderRadius: 24 },
  handle: { color: "#f5f5f7", fontSize: 22, fontWeight: "700" },
  hint: { color: "#9a9aa0", fontSize: 14 },
});
