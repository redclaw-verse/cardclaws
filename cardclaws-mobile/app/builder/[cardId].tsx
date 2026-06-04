import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Stack, useLocalSearchParams, useRouter } from "expo-router";
import { useEffect } from "react";
import { ActivityIndicator, Alert, Pressable, StyleSheet, Text, View } from "react-native";
import { getCard, publishCard, saveCard } from "../../src/api/cards";
import { BuilderCanvas } from "../../src/components/builder/BuilderCanvas";
import { useCardStore } from "../../src/stores/cardStore";

export default function BuilderScreen() {
  const { cardId } = useLocalSearchParams<{ cardId: string }>();
  const router = useRouter();
  const queryClient = useQueryClient();
  const load = useCardStore((s) => s.load);
  const storeCard = useCardStore((s) => s.card);

  const { data, isLoading } = useQuery({
    queryKey: ["card", cardId],
    queryFn: () => getCard(cardId),
    enabled: !!cardId,
  });

  useEffect(() => {
    if (data) load(data.definition);
  }, [data, load]);

  const save = useMutation({
    mutationFn: () => {
      const def = useCardStore.getState().card;
      if (!def) throw new Error("no card loaded");
      return saveCard(cardId, def);
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["cards"] }),
  });

  const publish = useMutation({
    mutationFn: async () => {
      const def = useCardStore.getState().card;
      if (def) await saveCard(cardId, def);
      return publishCard(cardId);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["cards"] });
      router.replace(`/card/${cardId}/view`);
    },
    onError: () => Alert.alert("Couldn't publish", "Check your plan's active-card limit."),
  });

  if (isLoading || !storeCard) {
    return (
      <View style={styles.center}>
        <ActivityIndicator color="#ff3b30" />
      </View>
    );
  }

  return (
    <View style={{ flex: 1 }}>
      <Stack.Screen
        options={{
          title: "Builder",
          headerRight: () => (
            <View style={styles.headerActions}>
              <Pressable onPress={() => save.mutate()} disabled={save.isPending}>
                <Text style={styles.headerBtn}>Save</Text>
              </Pressable>
              <Pressable onPress={() => publish.mutate()} disabled={publish.isPending}>
                <Text style={[styles.headerBtn, styles.publish]}>Publish</Text>
              </Pressable>
            </View>
          ),
        }}
      />
      <BuilderCanvas side="face" />
    </View>
  );
}

const styles = StyleSheet.create({
  center: { flex: 1, alignItems: "center", justifyContent: "center", backgroundColor: "#0a0a0c" },
  headerActions: { flexDirection: "row", gap: 16 },
  headerBtn: { color: "#f5f5f7", fontWeight: "600", fontSize: 16 },
  publish: { color: "#ff3b30" },
});
