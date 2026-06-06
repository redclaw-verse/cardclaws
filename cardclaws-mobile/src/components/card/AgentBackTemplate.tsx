// Agent card back: shown when an agent card is flipped. Renders the assistant's
// skill bars, tools, and special capabilities (trading-card style).

import { MaterialCommunityIcons } from "@expo/vector-icons";
import { useState } from "react";
import { Pressable, ScrollView, StyleSheet, Switch, Text, View } from "react-native";
import { AgentMeta } from "../../stores/localCardsStore";
import { QRCodeView } from "./QRCodeView";

export function AgentBackTemplate({
  name,
  title,
  agent,
}: {
  name: string;
  title: string;
  agent: AgentMeta;
}) {
  const [showQr, setShowQr] = useState(false);
  return (
    <View style={styles.root}>
      <View style={styles.header}>
        <Text style={styles.name} numberOfLines={1}>
          {name}
        </Text>
        {!!title && (
          <Text style={styles.title} numberOfLines={1}>
            {title}
          </Text>
        )}
        {!!agent.tagline && <Text style={styles.tagline}>{agent.tagline}</Text>}
      </View>

      <ScrollView contentContainerStyle={styles.body} showsVerticalScrollIndicator={false}>
        {agent.skills.length > 0 && (
          <>
            <Text style={styles.section}>Skills</Text>
            {agent.skills.map((s, i) => (
              <View key={`${s.name}-${i}`} style={styles.skillRow}>
                <Text style={styles.skillName} numberOfLines={1}>
                  {s.name}
                </Text>
                <View style={styles.bars}>
                  {[1, 2, 3, 4, 5].map((n) => (
                    <View key={n} style={[styles.bar, n <= s.level && styles.barOn]} />
                  ))}
                </View>
              </View>
            ))}
          </>
        )}

        {agent.tools.length > 0 && (
          <>
            <Text style={styles.section}>Tools</Text>
            <View style={styles.chips}>
              {agent.tools.map((t, i) => (
                <View key={`${t}-${i}`} style={styles.chip}>
                  <MaterialCommunityIcons name="wrench" size={13} color="#cbb8ff" />
                  <Text style={styles.chipText}>{t}</Text>
                </View>
              ))}
            </View>
          </>
        )}

        {agent.capabilities.length > 0 && (
          <>
            <Text style={styles.section}>Special capabilities</Text>
            {agent.capabilities.map((c, i) => (
              <View key={`${c}-${i}`} style={styles.capRow}>
                <MaterialCommunityIcons name="star-four-points" size={14} color="#ffd60a" />
                <Text style={styles.capText}>{c}</Text>
              </View>
            ))}
          </>
        )}

        {!!agent.brainUrl && (
          <View style={styles.shareBlock}>
            <Pressable style={styles.shareRow} onPress={() => setShowQr((v) => !v)}>
              <View style={{ flex: 1 }}>
                <Text style={styles.section}>Share brain link</Text>
                <Text style={styles.shareHint}>QR to access this brain on ClawBrainHub</Text>
              </View>
              <Switch
                value={showQr}
                onValueChange={setShowQr}
                trackColor={{ true: "#ff3b30", false: "#3a3a44" }}
                thumbColor="#ffffff"
              />
            </Pressable>
            {showQr && (
              <View style={styles.qrWrap}>
                <View style={styles.qrCard}>
                  <QRCodeView value={agent.brainUrl} size={150} />
                </View>
                <Text style={styles.qrUrl} numberOfLines={1}>
                  {agent.brainUrl}
                </Text>
              </View>
            )}
          </View>
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0e0e12", paddingHorizontal: 22, paddingVertical: 28 },
  header: { marginBottom: 12 },
  name: { color: "#ffffff", fontSize: 24, fontWeight: "800" },
  title: { color: "#9a9aa0", fontSize: 15, marginTop: 2 },
  tagline: { color: "#cbb8ff", fontSize: 13, marginTop: 6, fontStyle: "italic" },
  body: { gap: 8, paddingBottom: 16 },
  section: {
    color: "#9a9aa0",
    fontSize: 12,
    fontWeight: "700",
    textTransform: "uppercase",
    letterSpacing: 0.06 * 12,
    marginTop: 12,
  },
  skillRow: { flexDirection: "row", alignItems: "center", justifyContent: "space-between", gap: 12 },
  skillName: { color: "#f5f5f7", fontSize: 15, flex: 1 },
  bars: { flexDirection: "row", gap: 4 },
  bar: { width: 16, height: 8, borderRadius: 2, backgroundColor: "#26262e" },
  barOn: { backgroundColor: "#ff3b30" },
  chips: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  chip: {
    flexDirection: "row",
    alignItems: "center",
    gap: 5,
    backgroundColor: "#16161c",
    borderRadius: 10,
    paddingHorizontal: 10,
    paddingVertical: 7,
  },
  chipText: { color: "#f5f5f7", fontSize: 13, fontWeight: "600" },
  capRow: { flexDirection: "row", alignItems: "center", gap: 8 },
  capText: { color: "#e0e0e4", fontSize: 14, flex: 1 },
  shareBlock: {
    marginTop: 16,
    paddingTop: 14,
    borderTopWidth: StyleSheet.hairlineWidth,
    borderTopColor: "#2a2a30",
  },
  shareRow: { flexDirection: "row", alignItems: "center", gap: 12 },
  shareHint: { color: "#9a9aa0", fontSize: 12, marginTop: 2 },
  qrWrap: { alignItems: "center", marginTop: 14, gap: 8 },
  qrCard: { backgroundColor: "#ffffff", padding: 12, borderRadius: 14 },
  qrUrl: { color: "#9a9aa0", fontSize: 11 },
});

