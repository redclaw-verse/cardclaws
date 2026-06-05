// Editor for the back-of-card links (domain, GitHub, publications, services).
// Each link has a kind (sets its icon), a label, and a URL that becomes a
// scannable sub-QR on the card back.

import { MaterialCommunityIcons } from "@expo/vector-icons";
import { Pressable, StyleSheet, Text, TextInput, View } from "react-native";
import { newLayerId } from "../stores/cardStore";
import { CardLink, LinkKind } from "../stores/localCardsStore";

const KINDS: { kind: LinkKind; label: string; icon: keyof typeof MaterialCommunityIcons.glyphMap }[] =
  [
    { kind: "domain", label: "Domain", icon: "web" },
    { kind: "github", label: "GitHub", icon: "github" },
    { kind: "publication", label: "Paper", icon: "book-open-page-variant" },
    { kind: "link", label: "Link", icon: "link-variant" },
  ];

export function defaultLinks(): CardLink[] {
  return [
    { id: newLayerId(), kind: "domain", label: "Website", url: "https://cardclaws.com" },
    { id: newLayerId(), kind: "github", label: "GitHub", url: "https://github.com/redclawsystems" },
    {
      id: newLayerId(),
      kind: "publication",
      label: "Publications",
      url: "https://cardclaws.com/papers",
    },
  ];
}

export function LinksEditor({
  links,
  onChange,
}: {
  links: CardLink[];
  onChange: (links: CardLink[]) => void;
}) {
  const update = (id: string, patch: Partial<CardLink>) =>
    onChange(links.map((l) => (l.id === id ? { ...l, ...patch } : l)));
  const add = () => onChange([...links, { id: newLayerId(), kind: "link", label: "", url: "" }]);
  const remove = (id: string) => onChange(links.filter((l) => l.id !== id));

  return (
    <View style={styles.wrap}>
      {links.map((l) => (
        <View key={l.id} style={styles.card}>
          <View style={styles.kindRow}>
            {KINDS.map((k) => {
              const on = l.kind === k.kind;
              return (
                <Pressable
                  key={k.kind}
                  onPress={() => update(l.id, { kind: k.kind })}
                  style={[styles.kindChip, on && styles.kindOn]}
                >
                  <MaterialCommunityIcons name={k.icon} size={15} color={on ? "#fff" : "#9a9aa0"} />
                  <Text style={[styles.kindText, on && styles.kindTextOn]}>{k.label}</Text>
                </Pressable>
              );
            })}
          </View>
          <TextInput
            style={styles.input}
            placeholder="Label (e.g. GitHub)"
            placeholderTextColor="#6b6b70"
            value={l.label}
            onChangeText={(t) => update(l.id, { label: t })}
          />
          <TextInput
            style={styles.input}
            placeholder="https://…"
            placeholderTextColor="#6b6b70"
            autoCapitalize="none"
            autoCorrect={false}
            value={l.url}
            onChangeText={(t) => update(l.id, { url: t })}
          />
          <Pressable onPress={() => remove(l.id)} hitSlop={8}>
            <Text style={styles.remove}>Remove</Text>
          </Pressable>
        </View>
      ))}
      <Pressable style={styles.add} onPress={add}>
        <Text style={styles.addText}>+ Add link</Text>
      </Pressable>
    </View>
  );
}

const styles = StyleSheet.create({
  wrap: { gap: 12 },
  card: { backgroundColor: "#15151a", borderRadius: 14, padding: 12, gap: 8 },
  kindRow: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  kindChip: {
    flexDirection: "row",
    alignItems: "center",
    gap: 5,
    paddingHorizontal: 10,
    paddingVertical: 7,
    borderRadius: 10,
    backgroundColor: "#222228",
  },
  kindOn: { backgroundColor: "#ff3b30" },
  kindText: { color: "#9a9aa0", fontSize: 13, fontWeight: "600" },
  kindTextOn: { color: "#fff" },
  input: {
    backgroundColor: "#0e0e12",
    color: "#f5f5f7",
    borderRadius: 10,
    paddingHorizontal: 12,
    paddingVertical: 10,
    fontSize: 15,
  },
  remove: { color: "#ff453a", fontSize: 13, fontWeight: "600", paddingTop: 2 },
  add: {
    borderWidth: 1,
    borderColor: "#2a2a30",
    borderStyle: "dashed",
    borderRadius: 14,
    paddingVertical: 14,
    alignItems: "center",
  },
  addText: { color: "#f5f5f7", fontWeight: "600", fontSize: 15 },
});
