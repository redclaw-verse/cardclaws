// Organized back-of-card theme (PRD §6.3): identity header, a primary "scan to
// connect" QR, and a list of labeled sub-QR links (domain, GitHub, publications,
// any service) people can quickly scan.

import { MaterialCommunityIcons } from "@expo/vector-icons";
import { ScrollView, StyleSheet, Text, View } from "react-native";
import { CardLink, LinkKind } from "../../stores/localCardsStore";
import { QRCodeView } from "./QRCodeView";

const KIND_ICON: Record<LinkKind, keyof typeof MaterialCommunityIcons.glyphMap> = {
  domain: "web",
  github: "github",
  publication: "book-open-page-variant",
  link: "link-variant",
};

/** Strip the scheme/trailing slash for a compact display URL. */
function prettyUrl(url: string): string {
  return url.replace(/^https?:\/\//, "").replace(/\/$/, "");
}

export function CardBackTemplate({
  name,
  title,
  profileUrl,
  links,
}: {
  name: string;
  title: string;
  profileUrl: string;
  links: CardLink[];
}) {
  // Only show links that actually have a URL (empty ones would be blank QRs).
  const shown = links.filter((l) => l.url.trim());
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
      </View>

      <View style={styles.hero}>
        <View style={styles.heroQr}>
          <QRCodeView value={profileUrl} size={132} />
        </View>
        <Text style={styles.heroCaption}>Scan to view everything</Text>
        <Text style={styles.heroUrl} numberOfLines={1}>
          {prettyUrl(profileUrl)}
        </Text>
      </View>

      <View style={styles.divider} />

      <ScrollView
        style={styles.links}
        contentContainerStyle={styles.linksContent}
        showsVerticalScrollIndicator={false}
      >
        {shown.map((link) => (
          <View key={link.id} style={styles.row}>
            <View style={styles.iconWrap}>
              <MaterialCommunityIcons name={KIND_ICON[link.kind]} size={20} color="#f5f5f7" />
            </View>
            <View style={styles.rowText}>
              <Text style={styles.rowLabel} numberOfLines={1}>
                {link.label}
              </Text>
              <Text style={styles.rowUrl} numberOfLines={1}>
                {prettyUrl(link.url)}
              </Text>
            </View>
          </View>
        ))}
        {shown.length === 0 && (
          <Text style={styles.empty}>Add links in the editor to show scannable codes here.</Text>
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0e0e12", paddingHorizontal: 22, paddingVertical: 28 },
  header: { marginBottom: 16 },
  name: { color: "#ffffff", fontSize: 24, fontWeight: "800" },
  title: { color: "#9a9aa0", fontSize: 15, marginTop: 2 },
  hero: { alignItems: "center", marginBottom: 18 },
  heroQr: { backgroundColor: "#ffffff", borderRadius: 14, padding: 10 },
  heroCaption: { color: "#f5f5f7", fontSize: 15, fontWeight: "600", marginTop: 10 },
  heroUrl: { color: "#6b6b70", fontSize: 12, marginTop: 2 },
  divider: { height: StyleSheet.hairlineWidth, backgroundColor: "#2a2a30", marginBottom: 6 },
  links: { flex: 1 },
  linksContent: { gap: 10, paddingVertical: 8 },
  row: {
    flexDirection: "row",
    alignItems: "center",
    gap: 12,
    backgroundColor: "#16161c",
    borderRadius: 14,
    padding: 10,
  },
  iconWrap: {
    width: 40,
    height: 40,
    borderRadius: 12,
    backgroundColor: "#26262e",
    alignItems: "center",
    justifyContent: "center",
  },
  rowText: { flex: 1 },
  rowLabel: { color: "#f5f5f7", fontSize: 15, fontWeight: "700" },
  rowUrl: { color: "#8a8a90", fontSize: 12, marginTop: 1 },
  empty: { color: "#6b6b70", fontSize: 13, textAlign: "center", paddingVertical: 24 },
});
