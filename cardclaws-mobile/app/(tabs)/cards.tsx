import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useRouter } from "expo-router";
import { FlatList, Pressable, StyleSheet, Text, View } from "react-native";
import { createCard, listCards } from "../../src/api/cards";
import { useAuthStore } from "../../src/stores/authStore";
import { CardDefinition, DEFAULT_SETTINGS, emptySide } from "../../src/types/card";

function defaultCard(handle: string, ownerId: string): CardDefinition {
  return {
    id: "",
    ownerId,
    handle,
    version: 1,
    face: emptySide("#1b1b2f"),
    back: emptySide("#1b1b2f"),
    palette: {
      colors: [
        { name: "Ocean", hex: "#1b1b2f", role: "background" },
        { name: "Snow", hex: "#f5f5f7", role: "text" },
        { name: "Claw", hex: "#ff3b30", role: "accent" },
      ],
    },
    settings: DEFAULT_SETTINGS,
  };
}

export default function CardsScreen() {
  const router = useRouter();
  const queryClient = useQueryClient();
  const user = useAuthStore((s) => s.user);

  const { data: cards = [], isLoading } = useQuery({
    queryKey: ["cards"],
    queryFn: listCards,
  });

  const create = useMutation({
    mutationFn: () => {
      const suffix = Math.floor(Math.random() * 1e4).toString(36);
      const handle = `${(user?.handle ?? "card").slice(0, 22)}-${suffix}`;
      return createCard(handle, defaultCard(handle, user?.id ?? ""));
    },
    onSuccess: (card) => {
      queryClient.invalidateQueries({ queryKey: ["cards"] });
      router.push(`/builder/${card.id}`);
    },
  });

  return (
    <View style={styles.root}>
      <FlatList
        data={cards}
        keyExtractor={(c) => c.id}
        contentContainerStyle={styles.list}
        ListEmptyComponent={
          isLoading ? null : <Text style={styles.empty}>No cards yet. Create your first.</Text>
        }
        renderItem={({ item }) => (
          <Pressable style={styles.card} onPress={() => router.push(`/card/${item.id}/view`)}>
            <Text style={styles.handle}>@{item.handle}</Text>
            <Text style={styles.status}>{item.status}</Text>
            <Pressable onPress={() => router.push(`/builder/${item.id}`)} hitSlop={8}>
              <Text style={styles.edit}>Edit</Text>
            </Pressable>
          </Pressable>
        )}
      />
      <Pressable style={styles.fab} onPress={() => create.mutate()} disabled={create.isPending}>
        <Text style={styles.fabText}>+ New card</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  list: { padding: 16, gap: 12 },
  empty: { color: "#6b6b70", textAlign: "center", marginTop: 64 },
  card: {
    backgroundColor: "#15151a",
    borderRadius: 16,
    padding: 18,
    flexDirection: "row",
    alignItems: "center",
    gap: 12,
  },
  handle: { color: "#f5f5f7", fontSize: 18, fontWeight: "700", flex: 1 },
  status: { color: "#9a9aa0", textTransform: "uppercase", fontSize: 12 },
  edit: { color: "#ff3b30", fontWeight: "600" },
  fab: {
    position: "absolute",
    bottom: 24,
    alignSelf: "center",
    backgroundColor: "#ff3b30",
    borderRadius: 28,
    paddingHorizontal: 28,
    paddingVertical: 16,
  },
  fabText: { color: "#fff", fontWeight: "700", fontSize: 16 },
});
