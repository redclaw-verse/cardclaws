// Editor for an agent card's flip-side data: tagline, skill bars, tools, and
// special capabilities.

import { useState } from "react";
import { Pressable, StyleSheet, Text, TextInput, View } from "react-native";
import { AgentMeta } from "../stores/localCardsStore";

const splitCsv = (s: string) =>
  s
    .split(",")
    .map((x) => x.trim())
    .filter(Boolean);

export function AgentDetailsEditor({
  meta,
  onChange,
}: {
  meta: AgentMeta;
  onChange: (m: AgentMeta) => void;
}) {
  // Keep the comma-separated fields as raw text so separators are typeable.
  const [toolsText, setToolsText] = useState(meta.tools.join(", "));
  const [capsText, setCapsText] = useState(meta.capabilities.join(", "));
  const set = (patch: Partial<AgentMeta>) => onChange({ ...meta, ...patch });

  return (
    <View style={{ gap: 10 }}>
      <Text style={styles.label}>Tagline</Text>
      <TextInput
        style={styles.input}
        placeholder="e.g. Your always-on research copilot"
        placeholderTextColor="#6b6b70"
        value={meta.tagline}
        onChangeText={(t) => set({ tagline: t })}
      />

      <Text style={styles.label}>Skills</Text>
      {meta.skills.map((s, i) => (
        <View key={i} style={styles.skillRow}>
          <TextInput
            style={[styles.input, { flex: 1 }]}
            placeholder="Skill"
            placeholderTextColor="#6b6b70"
            value={s.name}
            onChangeText={(t) =>
              set({ skills: meta.skills.map((x, idx) => (idx === i ? { ...x, name: t } : x)) })
            }
          />
          <View style={styles.dots}>
            {[1, 2, 3, 4, 5].map((n) => (
              <Pressable
                key={n}
                onPress={() =>
                  set({ skills: meta.skills.map((x, idx) => (idx === i ? { ...x, level: n } : x)) })
                }
                style={[styles.dot, n <= s.level && styles.dotOn]}
              />
            ))}
          </View>
          <Pressable onPress={() => set({ skills: meta.skills.filter((_, idx) => idx !== i) })} hitSlop={8}>
            <Text style={styles.remove}>✕</Text>
          </Pressable>
        </View>
      ))}
      <Pressable
        style={styles.add}
        onPress={() => set({ skills: [...meta.skills, { name: "", level: 3 }] })}
      >
        <Text style={styles.addText}>+ Add skill</Text>
      </Pressable>

      <Text style={styles.label}>Tools (comma-separated)</Text>
      <TextInput
        style={styles.input}
        placeholder="e.g. Search, Code, Email"
        placeholderTextColor="#6b6b70"
        autoCapitalize="none"
        value={toolsText}
        onChangeText={(t) => {
          setToolsText(t);
          set({ tools: splitCsv(t) });
        }}
      />

      <Text style={styles.label}>Special capabilities (comma-separated)</Text>
      <TextInput
        style={[styles.input, { minHeight: 64, textAlignVertical: "top" }]}
        multiline
        placeholder="e.g. Multi-step planning, Long-term memory"
        placeholderTextColor="#6b6b70"
        value={capsText}
        onChangeText={(t) => {
          setCapsText(t);
          set({ capabilities: splitCsv(t) });
        }}
      />
    </View>
  );
}

const styles = StyleSheet.create({
  label: { color: "#9a9aa0", fontSize: 14, marginTop: 4 },
  input: {
    backgroundColor: "#15151a",
    color: "#f5f5f7",
    borderRadius: 12,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 16,
  },
  skillRow: { flexDirection: "row", alignItems: "center", gap: 10 },
  dots: { flexDirection: "row", gap: 4 },
  dot: { width: 14, height: 14, borderRadius: 7, backgroundColor: "#26262e" },
  dotOn: { backgroundColor: "#ff3b30" },
  remove: { color: "#ff453a", fontSize: 16, fontWeight: "700", paddingHorizontal: 4 },
  add: {
    borderWidth: 1,
    borderColor: "#2a2a30",
    borderStyle: "dashed",
    borderRadius: 12,
    paddingVertical: 12,
    alignItems: "center",
  },
  addText: { color: "#f5f5f7", fontWeight: "600", fontSize: 15 },
});
