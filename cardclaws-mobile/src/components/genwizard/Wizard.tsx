// A small multi-step "brief builder" used by the AI image + video generators.
// Each step collects one value; the final review composes a prompt. Generation
// itself is handled by the CardClaws AI service (Anthropic-powered, PRD §21
// Phase 5) — this screen produces the brief that drives it.

import { Stack, useRouter } from "expo-router";
import { ReactNode, useState } from "react";
import { ActivityIndicator, Alert, Pressable, ScrollView, StyleSheet, Text, View } from "react-native";

export interface WizardStep {
  key: string;
  title: string;
  subtitle?: string;
  render: (value: string, setValue: (v: string) => void) => ReactNode;
}

/** Single-select chip row. */
export function Chips({
  options,
  value,
  onSelect,
}: {
  options: string[];
  value: string;
  onSelect: (v: string) => void;
}) {
  return (
    <View style={styles.chips}>
      {options.map((o) => (
        <Pressable
          key={o}
          onPress={() => onSelect(o)}
          style={[styles.chip, value === o && styles.chipOn]}
        >
          <Text style={[styles.chipText, value === o && styles.chipTextOn]}>{o}</Text>
        </Pressable>
      ))}
    </View>
  );
}

/** Multi-select chip row (sibling of Chips; same styling). */
export function MultiChips({
  options,
  values,
  onToggle,
}: {
  options: string[];
  values: string[];
  onToggle: (v: string) => void;
}) {
  return (
    <View style={styles.chips}>
      {options.map((o) => {
        const on = values.includes(o);
        return (
          <Pressable key={o} onPress={() => onToggle(o)} style={[styles.chip, on && styles.chipOn]}>
            <Text style={[styles.chipText, on && styles.chipTextOn]}>{o}</Text>
          </Pressable>
        );
      })}
    </View>
  );
}

export function Wizard({
  title,
  steps,
  compose,
  generateLabel,
  onGenerate: onGenerateOverride,
}: {
  title: string;
  steps: WizardStep[];
  compose: (values: Record<string, string>) => string;
  generateLabel: string;
  /** Custom generate action (e.g. call the backend). Falls back to a summary. */
  onGenerate?: (values: Record<string, string>) => Promise<void>;
}) {
  const router = useRouter();
  const [index, setIndex] = useState(0);
  const [values, setValues] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);

  const reviewing = index >= steps.length;
  const step = steps[index];
  const setForStep = (v: string) => setValues((s) => ({ ...s, [step.key]: v }));

  const onGenerate = async () => {
    if (!onGenerateOverride) {
      Alert.alert(
        "Generating…",
        `${compose(values)}\n\nThis brief is sent to the CardClaws AI generator.`,
        [{ text: "Done", onPress: () => router.back() }],
      );
      return;
    }
    setBusy(true);
    try {
      await onGenerateOverride(values);
    } catch {
      Alert.alert("Couldn't generate", "Please try again in a moment.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <View style={styles.root}>
      <Stack.Screen options={{ title }} />

      <View style={styles.dots}>
        {steps.map((s, i) => (
          <View key={s.key} style={[styles.dot, i === index && styles.dotOn, i < index && styles.dotDone]} />
        ))}
        <View style={[styles.dot, reviewing && styles.dotOn]} />
      </View>

      <ScrollView contentContainerStyle={styles.content}>
        {reviewing ? (
          <>
            <Text style={styles.h}>Review</Text>
            <Text style={styles.subtitle}>We&apos;ll generate from this brief:</Text>
            <View style={styles.review}>
              <Text style={styles.reviewText}>{compose(values)}</Text>
            </View>
          </>
        ) : (
          <>
            <Text style={styles.h}>{step.title}</Text>
            {step.subtitle && <Text style={styles.subtitle}>{step.subtitle}</Text>}
            <View style={styles.body}>{step.render(values[step.key] ?? "", setForStep)}</View>
          </>
        )}
      </ScrollView>

      <View style={styles.footer}>
        {index > 0 && !busy && (
          <Pressable style={styles.back} onPress={() => setIndex(index - 1)}>
            <Text style={styles.backText}>Back</Text>
          </Pressable>
        )}
        <Pressable
          style={[styles.next, busy && styles.nextBusy]}
          disabled={busy}
          onPress={() => (reviewing ? onGenerate() : setIndex(index + 1))}
        >
          {busy ? (
            <ActivityIndicator color="#fff" />
          ) : (
            <Text style={styles.nextText}>{reviewing ? generateLabel : "Next"}</Text>
          )}
        </Pressable>
      </View>
    </View>
  );
}

export const wizardInputStyle = {
  backgroundColor: "#15151a",
  color: "#f5f5f7",
  borderRadius: 12,
  paddingHorizontal: 14,
  paddingVertical: 12,
  fontSize: 16,
  minHeight: 96,
  textAlignVertical: "top" as const,
};

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  dots: { flexDirection: "row", gap: 8, padding: 20, paddingBottom: 8 },
  dot: { flex: 1, height: 4, borderRadius: 2, backgroundColor: "#222228" },
  dotOn: { backgroundColor: "#ff3b30" },
  dotDone: { backgroundColor: "#7a2620" },
  content: { padding: 20, gap: 10 },
  h: { color: "#f5f5f7", fontSize: 26, fontWeight: "800" },
  subtitle: { color: "#9a9aa0", fontSize: 15 },
  body: { marginTop: 8 },
  chips: { flexDirection: "row", flexWrap: "wrap", gap: 10 },
  chip: {
    paddingHorizontal: 16,
    paddingVertical: 12,
    borderRadius: 14,
    backgroundColor: "#15151a",
    borderWidth: 1,
    borderColor: "#222228",
  },
  chipOn: { backgroundColor: "#ff3b30", borderColor: "#ff3b30" },
  chipText: { color: "#f5f5f7", fontSize: 15, fontWeight: "600" },
  chipTextOn: { color: "#fff" },
  review: { backgroundColor: "#15151a", borderRadius: 16, padding: 16 },
  reviewText: { color: "#f5f5f7", fontSize: 17, lineHeight: 24 },
  footer: { flexDirection: "row", gap: 12, padding: 20, paddingBottom: 36 },
  back: {
    backgroundColor: "#222228",
    borderRadius: 16,
    paddingVertical: 16,
    paddingHorizontal: 24,
    alignItems: "center",
  },
  backText: { color: "#f5f5f7", fontWeight: "600", fontSize: 16 },
  next: {
    flex: 1,
    backgroundColor: "#ff3b30",
    borderRadius: 16,
    paddingVertical: 16,
    alignItems: "center",
  },
  nextBusy: { opacity: 0.8 },
  nextText: { color: "#fff", fontWeight: "700", fontSize: 17 },
});
