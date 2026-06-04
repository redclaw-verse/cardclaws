import { Tabs } from "expo-router";

export default function TabsLayout() {
  return (
    <Tabs
      screenOptions={{
        headerStyle: { backgroundColor: "#0a0a0c" },
        headerTintColor: "#f5f5f7",
        tabBarStyle: { backgroundColor: "#0a0a0c", borderTopColor: "#1a1a1f" },
        tabBarActiveTintColor: "#ff3b30",
        tabBarInactiveTintColor: "#6b6b70",
      }}
    >
      <Tabs.Screen name="cards" options={{ title: "Cards" }} />
      <Tabs.Screen name="settings" options={{ title: "Settings" }} />
    </Tabs>
  );
}
