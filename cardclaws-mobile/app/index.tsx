// Entry redirect: authenticated users land on their cards, others on login.
import { Redirect } from "expo-router";
import { useAuthStore } from "../src/stores/authStore";

export default function Index() {
  const authed = useAuthStore((s) => s.accessToken !== null);
  return <Redirect href={authed ? "/(tabs)/cards" : "/(auth)/login"} />;
}
