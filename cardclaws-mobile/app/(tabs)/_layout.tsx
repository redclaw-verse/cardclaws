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
  const hydrated = useProfileStore((s) => s.hydrated);
  const onboarded = useProfileStore((s) => s.onboarded);

  // Wait for MMKV to rehydrate before deciding, then send first-run users to
  // onboarding (avoids a flash of the tabs).
  if (!hydrated) return null;
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
          paddingTop: 10,
          paddingBottom: Math.max(insets.bottom, 14),
        },
        tabBarLabelStyle: { fontSize: 11, fontWeight: "600", marginBottom: 2 },
        tabBarItemStyle: { paddingVertical: 4 },
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
