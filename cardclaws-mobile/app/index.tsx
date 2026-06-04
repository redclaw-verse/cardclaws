// Landing: jump straight into the no-login demo, or sign in to the full app.
import { useRouter } from "expo-router";
import { Pressable, StyleSheet, Text, View } from "react-native";

export default function Index() {
  const router = useRouter();
  return (
    <View style={styles.root}>
      <Text style={styles.title}>CardClaws</Text>
      <Text style={styles.tagline}>The other side of you.</Text>

      <Pressable style={styles.primary} onPress={() => router.push("/demo")}>
        <Text style={styles.primaryText}>Try the card demo</Text>
      </Pressable>
      <Pressable onPress={() => router.push("/(auth)/login")}>
        <Text style={styles.link}>Sign in to the full app</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c", alignItems: "center", justifyContent: "center", padding: 24, gap: 14 },
  title: { color: "#f5f5f7", fontSize: 44, fontWeight: "800", letterSpacing: -1 },
  tagline: { color: "#9a9aa0", marginBottom: 24 },
  primary: { backgroundColor: "#ff3b30", borderRadius: 16, paddingVertical: 16, paddingHorizontal: 40 },
  primaryText: { color: "#fff", fontWeight: "700", fontSize: 17 },
  link: { color: "#9a9aa0", marginTop: 12 },
});
