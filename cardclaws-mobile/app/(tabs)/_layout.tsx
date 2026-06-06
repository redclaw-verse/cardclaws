// Bottom tab bar for the standalone app:
//   backpack → your cards (gallery)
//   money bag → collectible cards
//   robot → work-agent cards
//   cog → settings
import { MaterialCommunityIcons } from "@expo/vector-icons";
import { Tabs } from "expo-router";

type IconName = keyof typeof MaterialCommunityIcons.glyphMap;

function tabIcon(name: IconName) {
  return ({ color, size }: { color: string; size: number }) => (
    <MaterialCommunityIcons name={name} size={size} color={color} />
  );
}

export default function TabsLayout() {
  return (
    <Tabs
      screenOptions={{
        headerShown: false,
        tabBarActiveTintColor: "#ff3b30",
        tabBarInactiveTintColor: "#6b6b70",
        tabBarStyle: {
          backgroundColor: "#0e0e12",
          borderTopColor: "#1c1c22",
        },
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
