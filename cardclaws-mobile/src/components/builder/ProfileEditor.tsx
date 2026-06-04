// Profile-depth editor (PRD §6.6): bio + links. Edits are undoable via the
// cardStore profile actions. Portfolio/testimonials are Pro-tier and land later.

import { Modal, Pressable, ScrollView, StyleSheet, Text, TextInput, View } from "react-native";
import { newLayerId, useCardStore } from "../../stores/cardStore";

interface Props {
  visible: boolean;
  onClose: () => void;
}

const MAX_LINKS = 12;

export function ProfileEditor({ visible, onClose }: Props) {
  const profile = useCardStore((s) => s.card?.profile);
  const setBio = useCardStore((s) => s.setBio);
  const addLink = useCardStore((s) => s.addLink);
  const updateLink = useCardStore((s) => s.updateLink);
  const removeLink = useCardStore((s) => s.removeLink);

  const links = profile?.links ?? [];

  return (
    <Modal visible={visible} transparent animationType="slide" onRequestClose={onClose}>
      <Pressable style={styles.backdrop} onPress={onClose}>
        <Pressable style={styles.sheet} onPress={(e) => e.stopPropagation()}>
          <View style={styles.grabber} />
          <ScrollView contentContainerStyle={styles.content}>
            <Text style={styles.title}>Profile</Text>

            <Text style={styles.label}>Bio</Text>
            <TextInput
              style={[styles.input, styles.bio]}
              multiline
              maxLength={400}
              placeholder="A short bio (up to 400 chars)…"
              placeholderTextColor="#6b6b70"
              value={profile?.bio ?? ""}
              onChangeText={setBio}
            />

            <View style={styles.linksHeader}>
              <Text style={styles.label}>Links ({links.length}/{MAX_LINKS})</Text>
              <Pressable
                disabled={links.length >= MAX_LINKS}
                onPress={() =>
                  addLink({ id: newLayerId(), type: "website", label: "", url: "", iconSlug: "website" })
                }
              >
                <Text style={[styles.add, links.length >= MAX_LINKS && styles.disabled]}>+ Add</Text>
              </Pressable>
            </View>

            {links.map((link) => (
              <View key={link.id} style={styles.linkRow}>
                <TextInput
                  style={[styles.input, styles.linkField]}
                  placeholder="Label"
                  placeholderTextColor="#6b6b70"
                  value={link.label}
                  onChangeText={(label) => updateLink(link.id, { label })}
                />
                <TextInput
                  style={[styles.input, styles.linkField]}
                  placeholder="https://…"
                  placeholderTextColor="#6b6b70"
                  autoCapitalize="none"
                  value={link.url}
                  onChangeText={(url) => updateLink(link.id, { url })}
                />
                <Pressable onPress={() => removeLink(link.id)} hitSlop={8}>
                  <Text style={styles.remove}>✕</Text>
                </Pressable>
              </View>
            ))}

            <Pressable style={styles.done} onPress={onClose}>
              <Text style={styles.doneText}>Done</Text>
            </Pressable>
          </ScrollView>
        </Pressable>
      </Pressable>
    </Modal>
  );
}

const styles = StyleSheet.create({
  backdrop: { flex: 1, backgroundColor: "rgba(0,0,0,0.5)", justifyContent: "flex-end" },
  sheet: {
    backgroundColor: "#15151a",
    borderTopLeftRadius: 24,
    borderTopRightRadius: 24,
    paddingTop: 12,
    maxHeight: "85%",
  },
  grabber: {
    alignSelf: "center",
    width: 40,
    height: 4,
    borderRadius: 2,
    backgroundColor: "#3a3a40",
    marginBottom: 8,
  },
  content: { padding: 20, paddingBottom: 36, gap: 10 },
  title: { color: "#f5f5f7", fontSize: 20, fontWeight: "700" },
  label: { color: "#9a9aa0", fontSize: 14 },
  input: {
    backgroundColor: "#222228",
    color: "#f5f5f7",
    borderRadius: 12,
    paddingHorizontal: 14,
    paddingVertical: 12,
  },
  bio: { minHeight: 80, textAlignVertical: "top" },
  linksHeader: { flexDirection: "row", justifyContent: "space-between", alignItems: "center", marginTop: 8 },
  add: { color: "#ff3b30", fontWeight: "600", fontSize: 16 },
  disabled: { opacity: 0.4 },
  linkRow: { flexDirection: "row", gap: 8, alignItems: "center" },
  linkField: { flex: 1 },
  remove: { color: "#9a9aa0", fontSize: 18, paddingHorizontal: 4 },
  done: { marginTop: 12, backgroundColor: "#ff3b30", borderRadius: 14, paddingVertical: 16, alignItems: "center" },
  doneText: { color: "#fff", fontWeight: "700", fontSize: 16 },
});
