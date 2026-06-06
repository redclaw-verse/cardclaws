// First-run onboarding: builds a rich user profile that personalizes AI assets
// and prefills business cards. Shown once (gated by profileStore.onboarded).

import { Redirect, Stack, useRouter } from "expo-router";
import { useState } from "react";
import { Pressable, ScrollView, StyleSheet, Text, TextInput, View } from "react-native";
import { useSafeAreaInsets } from "react-native-safe-area-context";
import { Chips, MultiChips, wizardInputStyle } from "../src/components/genwizard/Wizard";
import { LinksEditor } from "../src/components/LinksEditor";
import { CardLink } from "../src/stores/localCardsStore";
import {
  BRAND_COLOR_SWATCHES,
  EMPTY_PROFILE,
  Goal,
  GOAL_OPTIONS,
  INDUSTRY_OPTIONS,
  INSPIRATION_OPTIONS,
  Profile,
  VIBE_OPTIONS,
} from "../src/stores/profileLogic";
import { useProfileStore } from "../src/stores/profileStore";

const STEPS = 8;

export default function Onboarding() {
  const router = useRouter();
  const insets = useSafeAreaInsets();
  const onboarded = useProfileStore((s) => s.onboarded);
  const completeOnboarding = useProfileStore((s) => s.completeOnboarding);

  const [step, setStep] = useState(0);
  const [p, setP] = useState<Profile>(EMPTY_PROFILE);
  const patch = (u: Partial<Profile>) => setP((cur) => ({ ...cur, ...u }));
  const toggle = (key: "vibes" | "inspirations" | "brandColors", v: string, max?: number) =>
    setP((cur) => {
      const has = cur[key].includes(v);
      let next = has ? cur[key].filter((x) => x !== v) : [...cur[key], v];
      if (!has && max && next.length > max) next = next.slice(next.length - max);
      return { ...cur, [key]: next };
    });

  if (onboarded) return <Redirect href="/(tabs)" />;

  const canNext = step !== 0 || p.displayName.trim().length > 0;

  const finish = () => {
    completeOnboarding(p);
    router.replace("/(tabs)");
  };

  return (
    <View style={[styles.root, { paddingTop: insets.top + 8 }]}>
      <Stack.Screen options={{ headerShown: false }} />
      <View style={styles.dots}>
        {Array.from({ length: STEPS }).map((_, i) => (
          <View key={i} style={[styles.dot, i === step && styles.dotOn, i < step && styles.dotDone]} />
        ))}
      </View>

      <ScrollView contentContainerStyle={styles.content} keyboardShouldPersistTaps="handled">
        {step === 0 && (
          <Step title="Welcome 👋" subtitle="Let's set up your profile so the AI can tailor your cards.">
            <Text style={styles.label}>Your name</Text>
            <TextInput
              style={styles.input}
              placeholder="e.g. Omar Sobh"
              placeholderTextColor="#6b6b70"
              value={p.displayName}
              onChangeText={(t) => patch({ displayName: t })}
            />
            <Text style={styles.label}>Role / title</Text>
            <TextInput
              style={styles.input}
              placeholder="e.g. Founder & CEO"
              placeholderTextColor="#6b6b70"
              value={p.title}
              onChangeText={(t) => patch({ title: t })}
            />
          </Step>
        )}

        {step === 1 && (
          <Step title="Your industry" subtitle="Helps frame the look.">
            <Chips options={INDUSTRY_OPTIONS} value={p.industry} onSelect={(v) => patch({ industry: v })} />
          </Step>
        )}

        {step === 2 && (
          <Step title="Your aesthetic" subtitle="Pick the vibes you like — choose a few.">
            <MultiChips options={VIBE_OPTIONS} values={p.vibes} onToggle={(v) => toggle("vibes", v)} />
          </Step>
        )}

        {step === 3 && (
          <Step title="Primary goal" subtitle="What are these cards for?">
            <Chips
              options={GOAL_OPTIONS.map((g) => g.label)}
              value={GOAL_OPTIONS.find((g) => g.value === p.goal)?.label ?? ""}
              onSelect={(label) =>
                patch({ goal: (GOAL_OPTIONS.find((g) => g.label === label)?.value ?? "") as Goal })
              }
            />
          </Step>
        )}

        {step === 4 && (
          <Step title="Brand colors" subtitle="Pick up to 3 — we'll weave them into your art.">
            <View style={styles.swatches}>
              {BRAND_COLOR_SWATCHES.map((c) => (
                <Pressable
                  key={c}
                  onPress={() => toggle("brandColors", c, 3)}
                  style={[
                    styles.swatch,
                    { backgroundColor: c },
                    p.brandColors.includes(c) && styles.swatchOn,
                  ]}
                />
              ))}
            </View>
          </Step>
        )}

        {step === 5 && (
          <Step title="Social links" subtitle="Add any you want on your cards (optional).">
            <LinksEditor links={p.socialLinks} onChange={(l: CardLink[]) => patch({ socialLinks: l })} />
          </Step>
        )}

        {step === 6 && (
          <Step title="Inspiration" subtitle="Any looks you're drawn to? (up to 2)">
            <MultiChips
              options={INSPIRATION_OPTIONS}
              values={p.inspirations}
              onToggle={(v) => toggle("inspirations", v, 2)}
            />
          </Step>
        )}

        {step === 7 && (
          <Step title="You're set 🎉" subtitle="You can tweak any of this later in Settings.">
            <View style={styles.review}>
              <ReviewRow label="Name" value={p.displayName || "—"} />
              <ReviewRow label="Role" value={p.title || "—"} />
              <ReviewRow label="Industry" value={p.industry || "—"} />
              <ReviewRow label="Vibes" value={p.vibes.join(", ") || "—"} />
              <ReviewRow label="Goal" value={GOAL_OPTIONS.find((g) => g.value === p.goal)?.label ?? "—"} />
              <ReviewRow label="Colors" value={`${p.brandColors.length} picked`} />
              <ReviewRow label="Links" value={`${p.socialLinks.filter((l) => l.url).length}`} />
            </View>
          </Step>
        )}
      </ScrollView>

      <View style={[styles.footer, { paddingBottom: insets.bottom + 16 }]}>
        {step > 0 && (
          <Pressable style={styles.back} onPress={() => setStep(step - 1)}>
            <Text style={styles.backText}>Back</Text>
          </Pressable>
        )}
        <Pressable
          style={[styles.next, !canNext && styles.disabled]}
          disabled={!canNext}
          onPress={() => (step === STEPS - 1 ? finish() : setStep(step + 1))}
        >
          <Text style={styles.nextText}>{step === STEPS - 1 ? "Get started" : "Next"}</Text>
        </Pressable>
      </View>
    </View>
  );
}

function Step({ title, subtitle, children }: { title: string; subtitle: string; children: React.ReactNode }) {
  return (
    <View style={{ gap: 10 }}>
      <Text style={styles.h1}>{title}</Text>
      <Text style={styles.sub}>{subtitle}</Text>
      <View style={{ marginTop: 8 }}>{children}</View>
    </View>
  );
}

function ReviewRow({ label, value }: { label: string; value: string }) {
  return (
    <View style={styles.reviewRow}>
      <Text style={styles.reviewLabel}>{label}</Text>
      <Text style={styles.reviewValue} numberOfLines={1}>
        {value}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  dots: { flexDirection: "row", gap: 6, paddingHorizontal: 20, paddingBottom: 8 },
  dot: { flex: 1, height: 4, borderRadius: 2, backgroundColor: "#222228" },
  dotOn: { backgroundColor: "#ff3b30" },
  dotDone: { backgroundColor: "#7a2620" },
  content: { padding: 20, paddingBottom: 40 },
  h1: { color: "#f5f5f7", fontSize: 28, fontWeight: "800" },
  sub: { color: "#9a9aa0", fontSize: 15 },
  label: { color: "#9a9aa0", fontSize: 14, marginTop: 8 },
  input: { ...wizardInputStyle, minHeight: 0 },
  swatches: { flexDirection: "row", flexWrap: "wrap", gap: 14 },
  swatch: { width: 48, height: 48, borderRadius: 24, borderWidth: 3, borderColor: "transparent" },
  swatchOn: { borderColor: "#ffffff" },
  review: { backgroundColor: "#15151a", borderRadius: 16, paddingHorizontal: 16 },
  reviewRow: {
    flexDirection: "row",
    justifyContent: "space-between",
    paddingVertical: 12,
    borderBottomWidth: StyleSheet.hairlineWidth,
    borderBottomColor: "#22222a",
    gap: 16,
  },
  reviewLabel: { color: "#9a9aa0", fontSize: 14 },
  reviewValue: { color: "#f5f5f7", fontSize: 14, fontWeight: "600", flexShrink: 1 },
  footer: { flexDirection: "row", gap: 12, paddingHorizontal: 20, paddingTop: 12 },
  back: {
    backgroundColor: "#222228",
    borderRadius: 16,
    paddingVertical: 16,
    paddingHorizontal: 24,
    alignItems: "center",
  },
  backText: { color: "#f5f5f7", fontWeight: "600", fontSize: 16 },
  next: { flex: 1, backgroundColor: "#ff3b30", borderRadius: 16, paddingVertical: 16, alignItems: "center" },
  disabled: { opacity: 0.4 },
  nextText: { color: "#fff", fontWeight: "700", fontSize: 17 },
});
