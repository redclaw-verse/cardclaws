// Settings (standalone demo): app info + manage local data.
import { Alert, Pressable, StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useLocalCardsStore } from "../../src/stores/localCardsStore";

export default function SettingsScreen() {
  const insets = useSafeAreaInsets();
  const count = useLocalCardsStore((s) => s.cards.length);
  const clear = useLocalCardsStore((s) => s.clear);

  const confirmClear = () => {
    Alert.alert(
      "Clear all cards?",
      `This removes all ${count} card(s) on this device. This can't be undone.`,
      [
        { text: "Cancel", style: "cancel" },
        { text: "Clear", style: "destructive", onPress: () => clear() },
      ],
    );
  };

  return (
    <View style={[styles.root, { paddingTop: insets.top + 16 }]}>
      <Text style={styles.h1}>Settings</Text>

      <View style={styles.card}>
        <Row label="App" value="CardClaws" />
        <Row label="Mode" value="Standalone demo" />
        <Row label="Version" value="0.1.0" />
        <Row label="Cards on device" value={String(count)} />
      </View>

      <Pressable
        style={[styles.danger, count === 0 && styles.disabled]}
        disabled={count === 0}
        onPress={confirmClear}
      >
        <Text style={styles.dangerText}>Clear all cards</Text>
      </Pressable>
    </View>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <View style={styles.row}>
      <Text style={styles.label}>{label}</Text>
      <Text style={styles.value}>{value}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c", paddingHorizontal: 20, gap: 16 },
  h1: { color: "#f5f5f7", fontSize: 28, fontWeight: "800" },
  card: { backgroundColor: "#15151a", borderRadius: 16, paddingHorizontal: 16 },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    paddingVertical: 14,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: "#22222a",
  },
  label: { color: "#9a9aa0", fontSize: 15 },
  value: { color: "#f5f5f7", fontSize: 15, fontWeight: "600" },
  danger: {
    borderWidth: 1,
    borderColor: "#ff453a",
    borderRadius: 14,
    paddingVertical: 15,
    alignItems: "center",
  },
  disabled: { opacity: 0.4 },
  dangerText: { color: "#ff453a", fontWeight: "700", fontSize: 16 },
});
