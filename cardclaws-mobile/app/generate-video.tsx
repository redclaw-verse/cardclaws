// AI video generator (PRD §21 Phase 5) — single screen. Describe the motion,
// pick a style + length, then Generate: submit a Veo job to the backend, poll
// until the clip is ready, and drop the looping video onto the card front.

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
import { pollVideo, startVideo } from "../src/api/ai";
import { Chips, wizardInputStyle } from "../src/components/genwizard/Wizard";
import { useDraftStore } from "../src/stores/draftStore";
import { buildPersona } from "../src/stores/profileLogic";
import { useProfileStore } from "../src/stores/profileStore";

const MOTION = ["Parallax", "Particles", "Slow zoom", "Liquid", "Aurora", "Glitch"];
const LENGTH = ["3 seconds", "5 seconds", "8 seconds"];

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export default function GenerateVideo() {
  const router = useRouter();
  const setPendingVideoUri = useDraftStore((s) => s.setPendingVideoUri);
  const profile = useProfileStore((s) => s.profile);
  const [concept, setConcept] = useState("");
  const [motion, setMotion] = useState("");
  const [length, setLength] = useState("");
  const [busy, setBusy] = useState(false);

  const generate = async () => {
    if (!concept.trim()) {
      Alert.alert("Describe the motion", "Tell us how the card should come alive.");
      return;
    }
    const m = motion ? `${motion.toLowerCase()} ` : "";
    const len = length ? ` (${length})` : "";
    const persona = buildPersona(profile);
    const personaPrefix = persona
      ? `For a ${persona.role ?? "professional"}${persona.vibe ? ` (${persona.vibe} aesthetic)` : ""}: `
      : "";
    const prompt = `${personaPrefix}A ${m}motion clip for a digital business card: ${concept}${len}.`
      .replace(/\s+/g, " ")
      .trim();

    setBusy(true);
    try {
      const op = await startVideo(prompt);
      // Veo is async — poll until the clip is ready (typically ~1 minute).
      for (let i = 0; i < 40; i++) {
        const st = await pollVideo(op);
        if (st.status === "done" && st.videoBase64) {
          const path = `${FileSystem.cacheDirectory}veo-${Date.now()}.mp4`;
          await FileSystem.writeAsStringAsync(path, st.videoBase64, {
            encoding: FileSystem.EncodingType.Base64,
          });
          setPendingVideoUri(path);
          router.back();
          return;
        }
        if (st.status === "failed") throw new Error(st.error || "Video generation failed");
        await sleep(6000);
      }
      throw new Error("Timed out — try again in a moment.");
    } catch (e) {
      const msg = e instanceof Error ? e.message : "Please try again in a moment.";
      Alert.alert("Couldn't generate", msg);
    } finally {
      setBusy(false);
    }
  };

  return (
    <ScrollView style={styles.root} contentContainerStyle={styles.content}>
      <Stack.Screen options={{ title: "AI video" }} />

      <Text style={styles.label}>Describe the motion</Text>
      <TextInput
        style={wizardInputStyle}
        multiline
        editable={!busy}
        placeholder="e.g. slow drift across a city skyline as lights flicker on"
        placeholderTextColor="#6b6b70"
        value={concept}
        onChangeText={setConcept}
      />

      <Text style={styles.label}>Motion style</Text>
      <Chips options={MOTION} value={motion} onSelect={setMotion} />

      <Text style={styles.label}>Length</Text>
      <Chips options={LENGTH} value={length} onSelect={setLength} />

      <Pressable style={[styles.cta, busy && styles.ctaBusy]} disabled={busy} onPress={generate}>
        {busy ? <ActivityIndicator color="#fff" /> : <Text style={styles.ctaText}>Generate video</Text>}
      </Pressable>
      {busy && <Text style={styles.hint}>Generating with Veo — this takes about a minute…</Text>}
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
