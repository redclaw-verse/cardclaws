// Work-agent cards — AI agents attached to your cards. None yet; this is their
// home.
import { MaterialCommunityIcons } from "@expo/vector-icons";
import { StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";

export default function Agents() {
  const insets = useSafeAreaInsets();
  return (
    <View style={[styles.root, { paddingTop: insets.top }]}>
      <View style={styles.center}>
        <MaterialCommunityIcons name="robot" size={48} color="#3a3a44" />
        <Text style={styles.title}>No agent cards yet</Text>
        <Text style={styles.hint}>
          Cards backed by an AI work agent will appear here.
        </Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  center: { flex: 1, alignItems: "center", justifyContent: "center", padding: 32, gap: 10 },
  title: { color: "#f5f5f7", fontSize: 20, fontWeight: "700" },
  hint: { color: "#6b6b70", fontSize: 14, textAlign: "center", lineHeight: 20 },
});
