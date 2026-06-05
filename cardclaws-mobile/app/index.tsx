// Gallery of cards saved on this device (standalone demo). Tap a card to view /
// edit it, or create a new one. Falls back to a welcome state when empty.

import { useRouter } from "expo-router";
import { FlatList, Pressable, StyleSheet, Text, View } from "react-native";
import { CardThumb } from "../src/components/CardThumb";
import { LocalCard, useLocalCardsStore } from "../src/stores/localCardsStore";

/// Shrink tiles as the gallery grows: 2 columns → 4 fit a screen, 3 columns →
/// ~8 fit, 4 columns beyond that.
function columnsFor(count: number): number {
  if (count <= 4) return 2;
  if (count <= 9) return 3;
  return 4;
}

export default function Gallery() {
  const router = useRouter();
  const cards = useLocalCardsStore((s) => s.cards);
  const columns = columnsFor(cards.length);

  return (
    <View style={styles.root}>
      <View style={styles.header}>
        <Text style={styles.title}>Your cards</Text>
        <Pressable onPress={() => router.push("/(auth)/login")}>
          <Text style={styles.signin}>Sign in</Text>
        </Pressable>
      </View>

      <FlatList
        data={cards}
        keyExtractor={(c) => c.id}
        // numColumns can't change on the fly without remounting the list.
        key={`cols-${columns}`}
        numColumns={columns}
        columnWrapperStyle={cards.length > 0 ? styles.column : undefined}
        contentContainerStyle={styles.list}
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={styles.emptyTitle}>No cards yet</Text>
            <Text style={styles.emptyHint}>Tap “New card” to create your first.</Text>
          </View>
        }
        renderItem={({ item }: { item: LocalCard }) => (
          <Pressable style={styles.tile} onPress={() => router.push(`/demo?cardId=${item.id}`)}>
            <CardThumb item={item} style={styles.tileImage} />
            <View style={[styles.tileLabel, columns >= 3 && styles.tileLabelCompact]}>
              <Text style={[styles.tileName, columns >= 3 && styles.tileNameCompact]} numberOfLines={1}>
                {item.name}
              </Text>
              {!!item.title && columns < 4 && (
                <Text style={[styles.tileTitle, columns >= 3 && styles.tileNameCompact]} numberOfLines={1}>
                  {item.title}
                </Text>
              )}
            </View>
          </Pressable>
        )}
      />

      <Pressable style={styles.fab} onPress={() => router.push("/demo")}>
        <Text style={styles.fabText}>+ New card</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  header: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    paddingHorizontal: 20,
    paddingTop: 64,
    paddingBottom: 12,
  },
  title: { color: "#f5f5f7", fontSize: 30, fontWeight: "800" },
  signin: { color: "#9a9aa0", fontSize: 15 },
  list: { paddingHorizontal: 16, paddingBottom: 110, flexGrow: 1 },
  column: { gap: 12 },
  tile: { flex: 1, marginBottom: 12, borderRadius: 18, overflow: "hidden", backgroundColor: "#15151a" },
  tileImage: { width: "100%", aspectRatio: 2 / 3, overflow: "hidden" },
  tileLabel: { padding: 12 },
  tileLabelCompact: { padding: 7 },
  tileName: { color: "#f5f5f7", fontSize: 16, fontWeight: "700" },
  tileNameCompact: { fontSize: 12 },
  tileTitle: { color: "#9a9aa0", fontSize: 13, marginTop: 2 },
  empty: { flex: 1, alignItems: "center", justifyContent: "center", paddingTop: 120, gap: 6 },
  emptyTitle: { color: "#f5f5f7", fontSize: 20, fontWeight: "700" },
  emptyHint: { color: "#6b6b70" },
  fab: {
    position: "absolute",
    bottom: 28,
    alignSelf: "center",
    backgroundColor: "#ff3b30",
    borderRadius: 28,
    paddingHorizontal: 30,
    paddingVertical: 16,
  },
  fabText: { color: "#fff", fontWeight: "700", fontSize: 16 },
});
