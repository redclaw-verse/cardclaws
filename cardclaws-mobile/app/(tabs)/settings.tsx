import { useRouter } from "expo-router";
import { Pressable, StyleSheet, Text, View } from "react-native";
import { logout } from "../../src/api/auth";
import { useAuthStore } from "../../src/stores/authStore";

export default function SettingsScreen() {
  const router = useRouter();
  const user = useAuthStore((s) => s.user);

  const onLogout = async () => {
    await logout();
    router.replace("/(auth)/login");
  };

  return (
    <View style={styles.root}>
      <View style={styles.row}>
        <Text style={styles.label}>Signed in as</Text>
        <Text style={styles.value}>{user?.email ?? "—"}</Text>
      </View>
      <View style={styles.row}>
        <Text style={styles.label}>Handle</Text>
        <Text style={styles.value}>@{user?.handle ?? "—"}</Text>
      </View>
      <View style={styles.row}>
        <Text style={styles.label}>Plan</Text>
        <Text style={styles.value}>{user?.tier ?? "free"}</Text>
      </View>
      <Pressable style={styles.logout} onPress={onLogout}>
        <Text style={styles.logoutText}>Sign out</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c", padding: 16, gap: 4 },
  row: {
    flexDirection: "row",
    justifyContent: "space-between",
    paddingVertical: 16,
    borderBottomColor: "#1a1a1f",
    borderBottomWidth: 1,
  },
  label: { color: "#9a9aa0", fontSize: 15 },
  value: { color: "#f5f5f7", fontSize: 15, fontWeight: "600" },
  logout: {
    marginTop: 32,
    backgroundColor: "#15151a",
    borderRadius: 14,
    paddingVertical: 16,
    alignItems: "center",
  },
  logoutText: { color: "#ff453a", fontWeight: "700", fontSize: 16 },
});
