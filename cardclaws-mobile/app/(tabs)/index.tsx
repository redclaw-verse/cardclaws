// Cards (Backpack): your collection of business cards, with switchable views
// (Grid / Showcase) and an always-present "New card" entry.

import { useRouter } from "expo-router";
import { useState } from "react";
import { FlatList, Pressable, StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { CardThumb } from "../../src/components/CardThumb";
import { cardsOfKind, LocalCard, useLocalCardsStore } from "../../src/stores/localCardsStore";

type ViewMode = "grid" | "showcase";

/// Shrink grid tiles as the collection grows.
function columnsFor(count: number): number {
  if (count <= 4) return 2;
  if (count <= 9) return 3;
  return 4;
}

export default function Gallery() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const cards = cardsOfKind(
    useLocalCardsStore((s) => s.cards),
    "business",
  );
  const [view, setView] = useState<ViewMode>("grid");
  const columns = view === "showcase" ? 1 : columnsFor(cards.length);

  return (
    <View style={styles.root}>
      <View style={[styles.topBar, { paddingTop: insets.top + 10 }]}>
        <Segmented value={view} onChange={setView} />
      </View>

      <FlatList
        data={cards}
        keyExtractor={(c) => c.id}
        key={`${view}-${columns}`}
        numColumns={columns}
        columnWrapperStyle={columns > 1 ? styles.column : undefined}
        contentContainerStyle={styles.list}
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={styles.emptyTitle}>No cards yet</Text>
            <Text style={styles.emptyHint}>Tap “New card” to create your first.</Text>
          </View>
        }
        ListFooterComponent={
          <Pressable style={styles.addTile} onPress={() => router.push("/demo")}>
            <Text style={styles.addTileText}>+ New card</Text>
          </Pressable>
        }
        renderItem={({ item }: { item: LocalCard }) =>
          view === "showcase" ? (
            <Pressable style={styles.showTile} onPress={() => router.push(`/demo?cardId=${item.id}`)}>
              <CardThumb item={item} style={styles.showThumb} />
              <View style={styles.showOverlay}>
                <Text style={styles.showName} numberOfLines={1}>
                  {item.name}
                </Text>
                {!!item.title && (
                  <Text style={styles.showTitle} numberOfLines={1}>
                    {item.title}
                  </Text>
                )}
              </View>
            </Pressable>
          ) : (
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
          )
        }
      />

    </View>
  );
}

function Segmented({ value, onChange }: { value: ViewMode; onChange: (v: ViewMode) => void }) {
  return (
    <View style={styles.segment}>
      {(["grid", "showcase"] as const).map((v) => (
        <Pressable key={v} onPress={() => onChange(v)} style={[styles.segBtn, value === v && styles.segOn]}>
          <Text style={[styles.segText, value === v && styles.segTextOn]}>
            {v === "grid" ? "Grid" : "Showcase"}
          </Text>
        </Pressable>
      ))}
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  topBar: { paddingHorizontal: 16, paddingBottom: 10 },
  segment: { flexDirection: "row", backgroundColor: "#15151a", borderRadius: 12, padding: 4, gap: 4 },
  segBtn: { flex: 1, paddingVertical: 9, borderRadius: 9, alignItems: "center" },
  segOn: { backgroundColor: "#ff3b30" },
  segText: { color: "#9a9aa0", fontWeight: "600", fontSize: 14 },
  segTextOn: { color: "#fff" },
  list: { paddingHorizontal: 16, paddingBottom: 120, flexGrow: 1 },
  column: { gap: 12 },
  // Grid
  tile: { flex: 1, marginBottom: 12, borderRadius: 18, overflow: "hidden", backgroundColor: "#15151a" },
  tileImage: { width: "100%", aspectRatio: 2 / 3, overflow: "hidden" },
  tileLabel: { padding: 12 },
  tileLabelCompact: { padding: 7 },
  tileName: { color: "#f5f5f7", fontSize: 16, fontWeight: "700" },
  tileNameCompact: { fontSize: 12 },
  tileTitle: { color: "#9a9aa0", fontSize: 13, marginTop: 2 },
  // Showcase
  showTile: { marginBottom: 16, borderRadius: 22, overflow: "hidden", backgroundColor: "#15151a" },
  showThumb: { width: "100%", aspectRatio: 3 / 4, overflow: "hidden" },
  showOverlay: {
    position: "absolute",
    left: 0,
    right: 0,
    bottom: 0,
    padding: 18,
    backgroundColor: "rgba(10,10,12,0.72)",
  },
  showName: { color: "#fff", fontSize: 22, fontWeight: "800" },
  showTitle: { color: "#cfcfd4", fontSize: 14, marginTop: 2 },
  addTile: {
    borderWidth: 1,
    borderColor: "#2a2a30",
    borderStyle: "dashed",
    borderRadius: 22,
    paddingVertical: 28,
    alignItems: "center",
  },
  addTileText: { color: "#f5f5f7", fontWeight: "700", fontSize: 16 },
  empty: { flex: 1, alignItems: "center", justifyContent: "center", paddingTop: 120, gap: 6 },
  emptyTitle: { color: "#f5f5f7", fontSize: 20, fontWeight: "700" },
  emptyHint: { color: "#6b6b70" },
  fab: {
    position: "absolute",
    alignSelf: "center",
    backgroundColor: "#ff3b30",
    borderRadius: 28,
    paddingHorizontal: 30,
    paddingVertical: 16,
  },
  fabText: { color: "#fff", fontWeight: "700", fontSize: 16 },
});
