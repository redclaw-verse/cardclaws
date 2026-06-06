// Publish a saved demo card to the web (no login) — optionally generating an AI
// welcome shown when the QR is scanned. Returns a real cardclaws.com/<handle>
// URL and points the card's QR at it.

import { Stack, useLocalSearchParams, useRouter } from "expo-router";
import { useState } from "react";
import {
  ActivityIndicator,
  Alert,
  Linking,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import { publishToWeb } from "../src/api/ai";
import { QRCodeView } from "../src/components/card/QRCodeView";
import { wizardInputStyle } from "../src/components/genwizard/Wizard";
import { useLocalCardsStore } from "../src/stores/localCardsStore";

export default function PublishScreen() {
  const router = useRouter();
  const { cardId } = useLocalSearchParams<{ cardId?: string }>();
  const card = useLocalCardsStore((s) => (cardId ? s.getById(cardId) : undefined));
  const upsert = useLocalCardsStore((s) => s.upsert);

  const [welcomePrompt, setWelcomePrompt] = useState("");
  const [welcomeMessage, setWelcomeMessage] = useState("Great to meet you");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<string | null>(card?.publishedUrl ?? null);

  if (!card) {
    return (
      <ScrollView style={styles.root} contentContainerStyle={styles.content}>
        <Stack.Screen options={{ title: "Publish" }} />
        <Text style={styles.note}>Save the card first, then publish.</Text>
      </ScrollView>
    );
  }

  const publish = async () => {
    setBusy(true);
    try {
      const { profileUrl } = await publishToWeb({
        name: card.name,
        title: card.title || undefined,
        links: card.links.filter((l) => l.url.trim()).map((l) => ({ label: l.label, url: l.url })),
        welcomePrompt: welcomePrompt.trim() || undefined,
        welcomeMessage: welcomePrompt.trim() ? welcomeMessage : undefined,
      });
      // Point the card's QR at the real profile + remember it.
      upsert({ ...card, url: profileUrl, publishedUrl: profileUrl, updatedAt: Date.now() });
      setResult(profileUrl);
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Please try again in a moment.";
      Alert.alert("Couldn't publish", msg);
    } finally {
      setBusy(false);
    }
  };

  return (
    <ScrollView style={styles.root} contentContainerStyle={styles.content}>
      <Stack.Screen options={{ title: "Publish to web" }} />
      <Text style={styles.h1}>Publish “{card.name}”</Text>
      <Text style={styles.note}>
        Creates a real, scannable page at cardclaws.com — your card’s QR will point to it.
      </Text>

      <Text style={styles.label}>AI welcome scene (optional)</Text>
      <TextInput
        style={wizardInputStyle}
        multiline
        editable={!busy}
        placeholder="e.g. a calm misty redwood forest at dawn — plays when scanned"
        placeholderTextColor="#6b6b70"
        value={welcomePrompt}
        onChangeText={setWelcomePrompt}
      />

      {!!welcomePrompt.trim() && (
        <>
          <Text style={styles.label}>Welcome message</Text>
          <TextInput
            style={styles.input}
            editable={!busy}
            placeholder="Great to meet you"
            placeholderTextColor="#6b6b70"
            value={welcomeMessage}
            onChangeText={setWelcomeMessage}
          />
        </>
      )}

      <Pressable style={[styles.cta, busy && styles.ctaBusy]} disabled={busy} onPress={publish}>
        {busy ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text style={styles.ctaText}>{result ? "Re-publish" : "Publish to web"}</Text>
        )}
      </Pressable>
      {busy && !!welcomePrompt.trim() && (
        <Text style={styles.note}>Generating your AI welcome, then publishing…</Text>
      )}

      {result && (
        <>
          <Text style={styles.label}>Your live page — show this to scan</Text>
          <View style={styles.qrWrap}>
            <QRCodeView value={result} size={220} />
          </View>
          <Text style={styles.url} selectable>
            {result}
          </Text>
          <Pressable style={styles.secondary} onPress={() => Linking.openURL(result)}>
            <Text style={styles.secondaryText}>Open page</Text>
          </Pressable>
          <Pressable style={styles.secondary} onPress={() => router.replace("/")}>
            <Text style={styles.secondaryText}>Done</Text>
          </Pressable>
        </>
      )}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  content: { padding: 20, gap: 12 },
  h1: { color: "#f5f5f7", fontSize: 26, fontWeight: "800" },
  note: { color: "#9a9aa0", fontSize: 14 },
  label: { color: "#9a9aa0", fontSize: 15, marginTop: 8 },
  input: {
    backgroundColor: "#15151a",
    color: "#f5f5f7",
    borderRadius: 12,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 16,
  },
  cta: {
    backgroundColor: "#ff3b30",
    borderRadius: 16,
    paddingVertical: 16,
    alignItems: "center",
    marginTop: 12,
  },
  ctaBusy: { opacity: 0.8 },
  ctaText: { color: "#fff", fontWeight: "700", fontSize: 17 },
  qrWrap: {
    backgroundColor: "#ffffff",
    borderRadius: 16,
    padding: 14,
    alignSelf: "center",
  },
  url: { color: "#4da3ff", fontSize: 16 },
  secondary: {
    backgroundColor: "#222228",
    borderRadius: 14,
    paddingVertical: 14,
    alignItems: "center",
  },
  secondaryText: { color: "#f5f5f7", fontWeight: "600", fontSize: 16 },
});
