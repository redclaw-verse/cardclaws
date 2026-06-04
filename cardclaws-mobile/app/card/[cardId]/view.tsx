import { useQuery } from "@tanstack/react-query";
import { Stack, useLocalSearchParams } from "expo-router";
import { ActivityIndicator, View } from "react-native";
import { getCard } from "../../../src/api/cards";
import { CardViewer } from "../../../src/components/card/CardViewer";

export default function CardViewScreen() {
  const { cardId } = useLocalSearchParams<{ cardId: string }>();
  const { data, isLoading } = useQuery({
    queryKey: ["card", cardId],
    queryFn: () => getCard(cardId),
    enabled: !!cardId,
  });

  return (
    <View style={{ flex: 1, backgroundColor: "#0a0a0c" }}>
      <Stack.Screen options={{ title: "", headerTransparent: true }} />
      {isLoading || !data ? (
        <View style={{ flex: 1, alignItems: "center", justifyContent: "center" }}>
          <ActivityIndicator color="#ff3b30" />
        </View>
      ) : (
        <CardViewer card={data.definition} />
      )}
    </View>
  );
}
