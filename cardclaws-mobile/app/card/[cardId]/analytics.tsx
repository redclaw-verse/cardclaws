import { useQuery } from "@tanstack/react-query";
import { Stack, useLocalSearchParams } from "expo-router";
import { ActivityIndicator, ScrollView, StyleSheet, Text, View } from "react-native";
import { FeedEvent, GeoCount, getFeed, getGeo, getSummary } from "../../../src/api/analytics";

const EVENT_LABELS: Record<string, string> = {
  profile_visit: "Profile visit",
  qr_scan: "QR scan",
  nfc_tap: "NFC tap",
  contact_save: "Contact saved",
  link_click: "Link click",
  wallet_add: "Added to wallet",
};

export default function AnalyticsScreen() {
  const { cardId } = useLocalSearchParams<{ cardId: string }>();

  const summary = useQuery({
    queryKey: ["analytics", cardId, "summary"],
    queryFn: () => getSummary(cardId),
    enabled: !!cardId,
  });
  const feed = useQuery({
    queryKey: ["analytics", cardId, "feed"],
    queryFn: () => getFeed(cardId),
    enabled: !!cardId,
  });
  const geo = useQuery({
    queryKey: ["analytics", cardId, "geo"],
    queryFn: () => getGeo(cardId),
    enabled: !!cardId,
  });

  if (summary.isLoading) {
    return (
      <View style={styles.center}>
        <ActivityIndicator color="#ff3b30" />
      </View>
    );
  }

  const s = summary.data;

  return (
    <ScrollView style={styles.root} contentContainerStyle={styles.content}>
      <Stack.Screen options={{ title: "Analytics" }} />

      <View style={styles.metrics}>
        <Metric label="Visits (24h)" value={s?.visits24h ?? 0} />
        <Metric label="Visits (7d)" value={s?.visits7d ?? 0} />
        <Metric label="Total visits" value={s?.totalVisits ?? 0} />
        <Metric label="QR scans" value={s?.qrScans ?? 0} />
        <Metric label="Contact saves" value={s?.contactSaves ?? 0} />
        <Metric label="Link clicks" value={s?.linkClicks ?? 0} />
      </View>

      {geo.data && geo.data.length > 0 && (
        <Section title="Top countries">
          {geo.data.slice(0, 5).map((g: GeoCount) => (
            <Row key={g.country} left={g.country} right={`${g.visits}`} />
          ))}
        </Section>
      )}

      <Section title="Recent activity">
        {(feed.data ?? []).length === 0 ? (
          <Text style={styles.empty}>No activity yet.</Text>
        ) : (
          (feed.data ?? []).slice(0, 30).map((e: FeedEvent) => (
            <Row
              key={e.id}
              left={EVENT_LABELS[e.eventType] ?? e.eventType}
              right={[e.city, e.country].filter(Boolean).join(", ") || "—"}
            />
          ))
        )}
      </Section>
    </ScrollView>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <View style={styles.metric}>
      <Text style={styles.metricValue}>{value}</Text>
      <Text style={styles.metricLabel}>{label}</Text>
    </View>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <View style={styles.section}>
      <Text style={styles.sectionTitle}>{title}</Text>
      {children}
    </View>
  );
}

function Row({ left, right }: { left: string; right: string }) {
  return (
    <View style={styles.rowItem}>
      <Text style={styles.rowLeft}>{left}</Text>
      <Text style={styles.rowRight}>{right}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  content: { padding: 16, gap: 20 },
  center: { flex: 1, alignItems: "center", justifyContent: "center", backgroundColor: "#0a0a0c" },
  metrics: { flexDirection: "row", flexWrap: "wrap", gap: 12 },
  metric: {
    backgroundColor: "#15151a",
    borderRadius: 16,
    padding: 16,
    width: "47%",
  },
  metricValue: { color: "#f5f5f7", fontSize: 28, fontWeight: "800" },
  metricLabel: { color: "#9a9aa0", fontSize: 13, marginTop: 2 },
  section: { gap: 8 },
  sectionTitle: { color: "#f5f5f7", fontSize: 17, fontWeight: "700" },
  rowItem: {
    flexDirection: "row",
    justifyContent: "space-between",
    paddingVertical: 12,
    borderBottomColor: "#1a1a1f",
    borderBottomWidth: 1,
  },
  rowLeft: { color: "#f5f5f7", fontSize: 15 },
  rowRight: { color: "#9a9aa0", fontSize: 14 },
  empty: { color: "#6b6b70" },
});
