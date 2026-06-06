// Bottom tab bar for the standalone app:
//   backpack → your cards (gallery)
//   money bag → collectible cards
//   robot → work-agent cards
//   cog → settings
import { MaterialCommunityIcons } from "@expo/vector-icons";
import { Redirect, Tabs } from "expo-router";
import { StyleSheet } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { useProfileStore } from "../../src/stores/profileStore";

type IconName = keyof typeof MaterialCommunityIcons.glyphMap;

function tabIcon(name: IconName) {
  return ({ color, size }: { color: string; size: number }) => (
    <MaterialCommunityIcons name={name} size={size} color={color} />
  );
}

export default function TabsLayout() {
  const insets = useSafeAreaInsets();
  const onboarded = useProfileStore((s) => s.onboarded);

  // First-run users go to onboarding (MMKV hydrates synchronously, so this is
  // correct on the first render — no flash).
  if (!onboarded) return <Redirect href="/onboarding" />;

  return (
    <Tabs
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: "#ff3b30",
        tabBarInactiveTintColor: "#6b6b70",
        tabBarStyle: {
          backgroundColor: "#0e0e12",
          borderTopWidth: StyleSheet.hairlineWidth,
          borderTopColor: "#1c1c22",
          height: 64 + insets.bottom,
          paddingTop: 8,
          paddingBottom: insets.bottom + 10,
        },
        tabBarLabelStyle: { fontSize: 11, fontWeight: "600", marginBottom: 2 },
      }}
    >
      <Tabs.Screen name="index" options={{ title: "Cards", tabBarIcon: tabIcon("bag-personal") }} />
      <Tabs.Screen
        name="collectibles"
        options={{ title: "Collectibles", tabBarIcon: tabIcon("sack") }}
      />
      <Tabs.Screen name="agents" options={{ title: "Agents", tabBarIcon: tabIcon("robot") }} />
      <Tabs.Screen name="settings" options={{ title: "Settings", tabBarIcon: tabIcon("cog") }} />
    </Tabs>
  );
}
