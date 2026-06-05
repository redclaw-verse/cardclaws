// AI image scene generator (PRD §21 Phase 5) — single screen. Describe the
// scene, pick a style + mood, Generate: refine the brief with Gemini, create the
// image with nano-banana, and drop it onto the card front.

import * as FileSystem from "expo-file-system";
import { Stack, useRouter } from "expo-router";
import { useState } from "react";
import {
  ActivityIndicator,
  Alert,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
} from "react-native";
import { generateImage, refineScene } from "../src/api/ai";
import { Chips, wizardInputStyle } from "../src/components/genwizard/Wizard";
import { useDraftStore } from "../src/stores/draftStore";

const STYLES = ["Cinematic", "Minimal", "Neon", "Studio portrait", "Nature", "Abstract"];
const MOODS = ["Bold", "Calm", "Luxe", "Playful", "Dark"];

export default function GenerateImage() {
  const router = useRouter();
  const setPendingImageUri = useDraftStore((s) => s.setPendingImageUri);
  const [scene, setScene] = useState("");
  const [style, setStyle] = useState("");
  const [mood, setMood] = useState("");
  const [busy, setBusy] = useState(false);

  const generate = async () => {
    if (!scene.trim()) {
      Alert.alert("Describe your scene", "Tell us what the image should show.");
      return;
    }
    setBusy(true);
    try {
      const prompt = await refineScene({
        scene,
        style: style || undefined,
        mood: mood || undefined,
      });
      const img = await generateImage(prompt);
      const path = `${FileSystem.cacheDirectory}ai-${Date.now()}.png`;
      await FileSystem.writeAsStringAsync(path, img.imageBase64, {
        encoding: FileSystem.EncodingType.Base64,
      });
      setPendingImageUri(path);
      router.back();
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Please try again in a moment.";
      Alert.alert("Couldn't generate", msg);
    } finally {
      setBusy(false);
    }
  };

  return (
    <ScrollView style={styles.root} contentContainerStyle={styles.content}>
      <Stack.Screen options={{ title: "AI image" }} />

      <Text style={styles.label}>Describe your scene</Text>
      <TextInput
        style={wizardInputStyle}
        multiline
        placeholder="e.g. a neon-lit Tokyo street at night, rain on the pavement"
        placeholderTextColor="#6b6b70"
        value={scene}
        onChangeText={setScene}
      />

      <Text style={styles.label}>Style</Text>
      <Chips options={STYLES} value={style} onSelect={setStyle} />

      <Text style={styles.label}>Mood</Text>
      <Chips options={MOODS} value={mood} onSelect={setMood} />

      <Pressable style={[styles.cta, busy && styles.ctaBusy]} disabled={busy} onPress={generate}>
        {busy ? (
          <ActivityIndicator color="#fff" />
        ) : (
          <Text style={styles.ctaText}>Generate image</Text>
        )}
      </Pressable>
      {busy && <Text style={styles.hint}>Refining with Gemini, then creating your image…</Text>}
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  content: { padding: 20, gap: 12 },
  label: { color: "#9a9aa0", fontSize: 15, marginTop: 8 },
  cta: {
    backgroundColor: "#ff3b30",
    borderRadius: 16,
    paddingVertical: 16,
    alignItems: "center",
    marginTop: 20,
  },
  ctaBusy: { opacity: 0.8 },
  ctaText: { color: "#fff", fontWeight: "700", fontSize: 17 },
  hint: { color: "#9a9aa0", fontSize: 13, textAlign: "center", marginTop: 10 },
});
