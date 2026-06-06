// Reusable grid for a card family (Collectibles / Agents): responsive columns,
// an empty state, and a "+ New" tile below the cards that starts the creator
// pre-set to that kind.

import { useRouter } from "expo-router";
import { FlatList, Pressable, StyleSheet, Text, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { CardKind, cardsOfKind, LocalCard, useLocalCardsStore } from "../stores/localCardsStore";
import { CardThumb } from "./CardThumb";

function columnsFor(count: number): number {
  if (count <= 4) return 2;
  if (count <= 9) return 3;
  return 4;
}

export function CardCollection({
  kind,
  heading,
  addLabel,
  emptyTitle,
  emptyHint,
}: {
  kind: CardKind;
  heading: string;
  addLabel: string;
  emptyTitle: string;
  emptyHint: string;
}) {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const cards = cardsOfKind(
    useLocalCardsStore((s) => s.cards),
    kind,
  );
  const columns = columnsFor(cards.length);
  const openNew = () => router.push({ pathname: "/demo", params: { kind } });

  return (
    <View style={styles.root}>
      <View style={[styles.topBar, { paddingTop: insets.top + 14 }]}>
        <Text style={styles.heading}>{heading}</Text>
      </View>
      <FlatList
        data={cards}
        keyExtractor={(c) => c.id}
        key={`cols-${columns}`}
        numColumns={columns}
        columnWrapperStyle={columns > 1 ? styles.column : undefined}
        contentContainerStyle={styles.list}
        ListEmptyComponent={
          <View style={styles.empty}>
            <Text style={styles.emptyTitle}>{emptyTitle}</Text>
            <Text style={styles.emptyHint}>{emptyHint}</Text>
          </View>
        }
        ListFooterComponent={
          <Pressable style={styles.addTile} onPress={openNew}>
            <Text style={styles.addText}>{addLabel}</Text>
          </Pressable>
        }
        renderItem={({ item }: { item: LocalCard }) => (
          <Pressable
            style={styles.tile}
            onPress={() => router.push({ pathname: "/demo", params: { cardId: item.id } })}
          >
            <CardThumb item={item} style={styles.tileImage} />
            <View style={styles.tileLabel}>
              <Text style={styles.tileName} numberOfLines={1}>
                {item.name}
              </Text>
              {!!item.title && (
                <Text style={styles.tileTitle} numberOfLines={1}>
                  {item.title}
                </Text>
              )}
            </View>
          </Pressable>
        )}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  topBar: { paddingHorizontal: 16, paddingBottom: 10 },
  heading: { color: "#f5f5f7", fontSize: 26, fontWeight: "800" },
  list: { paddingHorizontal: 16, paddingBottom: 24, flexGrow: 1 },
  column: { gap: 12 },
  tile: { flex: 1, marginBottom: 12, borderRadius: 18, overflow: "hidden", backgroundColor: "#15151a" },
  tileImage: { width: "100%", aspectRatio: 2 / 3, overflow: "hidden" },
  tileLabel: { padding: 10 },
  tileName: { color: "#f5f5f7", fontSize: 14, fontWeight: "700" },
  tileTitle: { color: "#9a9aa0", fontSize: 12, marginTop: 2 },
  addTile: {
    borderWidth: 1,
    borderColor: "#2a2a30",
    borderStyle: "dashed",
    borderRadius: 18,
    paddingVertical: 22,
    alignItems: "center",
    marginTop: 4,
  },
  addText: { color: "#f5f5f7", fontWeight: "700", fontSize: 16 },
  empty: { alignItems: "center", justifyContent: "center", paddingVertical: 80, gap: 6 },
  emptyTitle: { color: "#f5f5f7", fontSize: 20, fontWeight: "700" },
  emptyHint: { color: "#6b6b70", fontSize: 14, textAlign: "center", paddingHorizontal: 24 },
});
