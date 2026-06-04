import { useQuery } from "@tanstack/react-query";
import { Stack, useLocalSearchParams, useRouter } from "expo-router";
import { useState } from "react";
import { ActivityIndicator, Pressable, StyleSheet, Text, View } from "react-native";
import { getCard } from "../../../src/api/cards";
import { CardViewer } from "../../../src/components/card/CardViewer";
import { ShareSheet } from "../../../src/components/share/ShareSheet";

export default function CardViewScreen() {
  const { cardId } = useLocalSearchParams<{ cardId: string }>();
  const router = useRouter();
  const [sharing, setSharing] = useState(false);

  const { data, isLoading } = useQuery({
    queryKey: ["card", cardId],
    queryFn: () => getCard(cardId),
    enabled: !!cardId,
  });

  return (
    <View style={{ flex: 1, backgroundColor: "#0a0a0c" }}>
      <Stack.Screen
        options={{
          title: "",
          headerTransparent: true,
          headerRight: () => (
            <View style={styles.actions}>
              <Pressable onPress={() => router.push(`/card/${cardId}/analytics`)}>
                <Text style={styles.headerBtn}>Stats</Text>
              </Pressable>
              <Pressable onPress={() => setSharing(true)}>
                <Text style={[styles.headerBtn, styles.share]}>Share</Text>
              </Pressable>
            </View>
          ),
        }}
      />
      {isLoading || !data ? (
        <View style={styles.center}>
          <ActivityIndicator color="#ff3b30" />
        </View>
      ) : (
        <>
          <CardViewer card={data.definition} />
          <ShareSheet
            cardId={cardId}
            handle={data.handle}
            visible={sharing}
            onClose={() => setSharing(false)}
          />
        </>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  center: { flex: 1, alignItems: "center", justifyContent: "center" },
  actions: { flexDirection: "row", gap: 16 },
  headerBtn: { color: "#f5f5f7", fontWeight: "600", fontSize: 16 },
  share: { color: "#ff3b30" },
});
