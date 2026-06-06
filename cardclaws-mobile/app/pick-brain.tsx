// Pick an agent brain from ClawBrainHub (your account + reference brains), pull
// it, and hand it to the agent-card editor to turn into a trading card.

import { Stack, useRouter } from "expo-router";
import { useEffect, useState } from "react";
import { ActivityIndicator, FlatList, Pressable, StyleSheet, Text, View } from "react-native";
import { BrainSummary, listBrains, pullBrain } from "../src/api/brainhub";
import { useDraftStore } from "../src/stores/draftStore";

export default function PickBrain() {
  const router = useRouter();
  const setPendingAgentBrain = useDraftStore((s) => s.setPendingAgentBrain);
  const [brains, setBrains] = useState<BrainSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [pulling, setPulling] = useState<string | null>(null);

  useEffect(() => {
    let alive = true;
    listBrains()
      .then((b) => alive && setBrains(b))
      .catch(() => alive && setError("Couldn't reach ClawBrainHub. Check your connection."))
      .finally(() => alive && setLoading(false));
    return () => {
      alive = false;
    };
  }, []);

  const choose = async (b: BrainSummary) => {
    const key = `${b.owner}/${b.name}`;
    setPulling(key);
    try {
      const brain = await pullBrain(b.owner, b.name, b.version);
      setPendingAgentBrain(brain);
      router.replace({ pathname: "/demo", params: { kind: "agent" } });
    } catch {
      setError("Couldn't pull that brain. Try another.");
      setPulling(null);
    }
  };

  return (
    <View style={styles.root}>
      <Stack.Screen options={{ title: "Pick an agent brain", headerTitleAlign: "center" }} />
      {loading ? (
        <View style={styles.center}>
          <ActivityIndicator color="#ff3b30" />
          <Text style={styles.dim}>Loading brains…</Text>
        </View>
      ) : error ? (
        <View style={styles.center}>
          <Text style={styles.error}>{error}</Text>
        </View>
      ) : (
        <FlatList
          data={brains}
          keyExtractor={(b) => `${b.owner}/${b.name}`}
          contentContainerStyle={styles.list}
          ListHeaderComponent={
            <Text style={styles.hint}>Pull a brain from ClawBrainHub to make an agent card.</Text>
          }
          renderItem={({ item }) => {
            const key = `${item.owner}/${item.name}`;
            return (
              <Pressable
                style={styles.row}
                disabled={!!pulling}
                onPress={() => choose(item)}
              >
                <View style={{ flex: 1 }}>
                  <Text style={styles.name} numberOfLines={1}>
                    {item.name}
                  </Text>
                  <Text style={styles.owner} numberOfLines={1}>
                    @{item.owner} · v{item.version}
                    {item.badge ? ` · ${item.badge}` : ""}
                  </Text>
                </View>
                {pulling === key ? (
                  <ActivityIndicator color="#ff3b30" />
                ) : (
                  <Text style={styles.pull}>Pull →</Text>
                )}
              </Pressable>
            );
          }}
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  center: { flex: 1, alignItems: "center", justifyContent: "center", gap: 12, padding: 32 },
  dim: { color: "#9a9aa0" },
  error: { color: "#ff453a", textAlign: "center", fontSize: 15 },
  list: { padding: 16, gap: 10 },
  hint: { color: "#9a9aa0", fontSize: 14, marginBottom: 8 },
  row: {
    flexDirection: "row",
    alignItems: "center",
    gap: 12,
    backgroundColor: "#15151a",
    borderRadius: 14,
    paddingHorizontal: 16,
    paddingVertical: 16,
  },
  name: { color: "#f5f5f7", fontSize: 16, fontWeight: "700" },
  owner: { color: "#9a9aa0", fontSize: 13, marginTop: 2 },
  pull: { color: "#ff3b30", fontWeight: "700", fontSize: 14 },
});
