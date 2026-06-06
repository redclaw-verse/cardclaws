// Standalone card editor (no login/backend): take/pick a photo → card front;
// fill name/title/QR link → Save. Saved cards live in the on-device gallery
// (localCardsStore). Opening with ?cardId=… edits an existing card.

import { MaterialCommunityIcons } from "@expo/vector-icons";
import { ResizeMode, Video } from "expo-av";
import * as ImagePicker from "expo-image-picker";
import { Stack, useFocusEffect, useLocalSearchParams, useRouter } from "expo-router";
import { StatusBar } from "expo-status-bar";
import { useCallback, useEffect, useState } from "react";
import {
  ActivityIndicator,
  Image,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import { AgentDetailsEditor } from "../src/components/AgentDetailsEditor";
import { AgentBackTemplate } from "../src/components/card/AgentBackTemplate";
import { CardBackTemplate } from "../src/components/card/CardBackTemplate";
import { CardViewer } from "../src/components/card/CardViewer";
import { defaultLinks } from "../src/components/LinksEditor";
import { newLayerId } from "../src/stores/cardStore";
import { useDraftStore } from "../src/stores/draftStore";
import {
  AgentMeta,
  CardKind,
  CardLink,
  Provenance,
  useLocalCardsStore,
} from "../src/stores/localCardsStore";
import { useProfileStore } from "../src/stores/profileStore";
import { mintProvenance } from "../src/utils/provenance";
import { CardDefinition, DEFAULT_SETTINGS, TextLayer } from "../src/types/card";
import { deleteImage, persistImage, persistVideo } from "../src/utils/imageStore";

function textLayer(
  text: string,
  y: number,
  size: number,
  color: string,
  align: "left" | "right" = "left",
): TextLayer {
  return {
    id: newLayerId(),
    type: "text",
    x: 0.08,
    y,
    width: 0.84,
    height: 0.12,
    opacity: 1,
    zIndex: 10,
    text,
    fontFamily: "System",
    fontWeight: 700,
    fontSize: size,
    lineHeight: size + 4,
    letterSpacing: 0,
    color,
    align,
  };
}

function buildDemoCard(
  mediaUri: string,
  isVideo: boolean,
  name: string,
  title: string,
): CardDefinition {
  return {
    id: "demo",
    ownerId: "demo",
    handle: "demo",
    version: 1,
    face: {
      // Name + title stacked at the top-right.
      layers: [
        textLayer(name, 0.06, 28, "#ffffff", "right"),
        title ? textLayer(title, 0.135, 17, "#f0f0f2", "right") : textLayer("", 0.135, 1, "#fff"),
      ],
      background: { type: isVideo ? "video" : "image", value: mediaUri },
      entryAnimation: "fade",
    },
    back: {
      layers: [
        {
          id: newLayerId(),
          type: "qr",
          x: 0.27,
          y: 0.14,
          width: 0.46,
          height: 0.31,
          opacity: 1,
          zIndex: 5,
        },
        textLayer(name, 0.58, 26, "#f5f5f7"),
        title ? textLayer(title, 0.67, 17, "#9a9aa0") : textLayer("", 0.67, 1, "#fff"),
      ],
      background: { type: "solid", value: "#101014" },
      entryAnimation: "fade",
    },
    palette: { colors: [] },
    settings: DEFAULT_SETTINGS,
  };
}

/** "general-assistant" → "General Assistant". */
function prettifyBrainName(name: string): string {
  return name
    .split(/[-_\s]+/)
    .filter(Boolean)
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(" ");
}

export default function CardEditorScreen() {
  const router = useRouter();
  const { cardId, kind: kindParam } = useLocalSearchParams<{ cardId?: string; kind?: string }>();
  const upsert = useLocalCardsStore((s) => s.upsert);
  const remove = useLocalCardsStore((s) => s.remove);
  const getById = useLocalCardsStore((s) => s.getById);

  // A stable id for the lifetime of this editor (new card or the one we're editing).
  // New cards prefill from the user's profile (so there's no data entry); the
  // edit path (cardId) overrides these in the effect below.
  const profile = useProfileStore((s) => s.profile);
  const [id] = useState(() => cardId ?? newLayerId());
  const [imageUri, setImageUri] = useState<string | null>(null);
  const [videoUri, setVideoUri] = useState<string | null>(null);
  const [name, setName] = useState(() => (cardId ? "" : profile.displayName));
  const [title, setTitle] = useState(() => (cardId ? "" : profile.title));
  const [url, setUrl] = useState("");
  const [links, setLinks] = useState<CardLink[]>(() =>
    cardId ? [] : profile.socialLinks.length > 0 ? profile.socialLinks : defaultLinks(),
  );
  const [cardKind, setCardKind] = useState<CardKind>(() => (kindParam as CardKind) || "business");
  const [agentMeta, setAgentMeta] = useState<AgentMeta>(() => ({
    tagline: "",
    skills: [],
    tools: [],
    capabilities: [],
  }));
  const [provenance, setProvenance] = useState<Provenance | undefined>(undefined);
  const [showing, setShowing] = useState(false);
  const [onBack, setOnBack] = useState(false);
  const [saving, setSaving] = useState(false);

  // Load an existing card and jump straight to viewing it.
  useEffect(() => {
    if (!cardId) return;
    const existing = getById(cardId);
    if (existing) {
      setImageUri(existing.imagePath || null);
      setVideoUri(existing.videoPath ?? null);
      setName(existing.name);
      setTitle(existing.title);
      setUrl(existing.url);
      setLinks(existing.links ?? []);
      setCardKind(existing.kind ?? "business");
      setProvenance(existing.provenance);
      if (existing.agent) setAgentMeta(existing.agent);
      setOnBack(false);
      setShowing(true);
    }
  }, [cardId, getById]);

  // When returning from the AI generators, adopt the image/video they produced.
  useFocusEffect(
    useCallback(() => {
      const draft = useDraftStore.getState();
      if (draft.pendingImageUri) {
        setImageUri(draft.pendingImageUri);
        setVideoUri(null);
        setShowing(false);
        draft.setPendingImageUri(null);
      }
      if (draft.pendingVideoUri) {
        setVideoUri(draft.pendingVideoUri);
        setShowing(false);
        draft.setPendingVideoUri(null);
      }
      // A brain pulled from ClawBrainHub → prefill the agent card's flip side.
      if (draft.pendingAgentBrain) {
        const b = draft.pendingAgentBrain;
        setCardKind("agent");
        setName(prettifyBrainName(b.name));
        setTitle(`v${b.version} · @${b.owner}`);
        setAgentMeta({
          tagline: b.tagline,
          skills: b.skills.map((s) => ({ name: s, level: 4 })),
          tools: b.tools,
          capabilities: b.capabilities,
        });
        setShowing(false);
        draft.setPendingAgentBrain(null);
      }
    }, []),
  );

  const pickFromLibrary = async () => {
    const res = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: [2, 3],
      quality: 0.9,
    });
    // A photo replaces any AI video on the front.
    if (!res.canceled) {
      setImageUri(res.assets[0].uri);
      setVideoUri(null);
    }
  };

  const takePhoto = async () => {
    const perm = await ImagePicker.requestCameraPermissionsAsync();
    if (!perm.granted) return;
    const res = await ImagePicker.launchCameraAsync({
      allowsEditing: true,
      aspect: [2, 3],
      quality: 0.9,
    });
    if (!res.canceled) {
      setImageUri(res.assets[0].uri);
      setVideoUri(null);
    }
  };

  // Persist the current edits to the local gallery (shared by Save + Publish).
  const persistCard = async () => {
    const existing = getById(id);
    const imagePath = imageUri ? await persistImage(imageUri, id) : (existing?.imagePath ?? "");
    const videoPath = videoUri ? await persistVideo(videoUri, id) : undefined;
    // Collectibles are "minted" (provenance recorded, mapped to the creator) on
    // acceptance — once; agents carry their flip-side metadata.
    const prov =
      cardKind === "collectible"
        ? (existing?.provenance ?? mintProvenance(profile.displayName || name))
        : undefined;
    const agent = cardKind === "agent" ? agentMeta : undefined;
    upsert({
      id,
      name,
      title,
      url,
      kind: cardKind,
      imagePath,
      videoPath,
      links,
      publishedUrl: existing?.publishedUrl,
      provenance: prov,
      agent,
      updatedAt: Date.now(),
    });
    setProvenance(prov);
    if (imageUri) setImageUri(imagePath);
    if (videoPath) setVideoUri(videoPath);
  };

  const save = async () => {
    if (!imageUri && !videoUri) return;
    setSaving(true);
    try {
      await persistCard();
      setOnBack(false);
      setShowing(true);
    } finally {
      setSaving(false);
    }
  };

  const goPublish = async () => {
    setSaving(true);
    try {
      await persistCard();
    } finally {
      setSaving(false);
    }
    router.push(`/publish?cardId=${id}`);
  };

  const onDelete = async () => {
    const existing = getById(id);
    if (existing?.imagePath) await deleteImage(existing.imagePath);
    if (existing?.videoPath) await deleteImage(existing.videoPath);
    remove(id);
    router.replace("/");
  };

  const media = videoUri ?? imageUri;
  if (showing && media) {
    const card = buildDemoCard(media, !!videoUri, name, title);
    return (
      <View style={styles.viewerRoot}>
        <Stack.Screen options={{ headerShown: false }} />
        <StatusBar hidden />
        <CardViewer
          card={card}
          profileUrl={url}
          fullScreen
          onSideChange={setOnBack}
          backContent={
            cardKind === "agent" ? (
              <AgentBackTemplate name={name} title={title} agent={agentMeta} />
            ) : (
              <CardBackTemplate
                name={name}
                title={title}
                profileUrl={url}
                links={links}
                provenance={cardKind === "collectible" ? provenance : undefined}
              />
            )
          }
        />
        {/* Front: a hint to flip. Back (QR side): the Gallery / Edit controls. */}
        {!onBack ? (
          <Text style={styles.swipeHint}>Swipe or tap the card to flip →</Text>
        ) : (
          <View style={styles.viewerActions}>
            <Pressable style={styles.pillDark} onPress={() => router.replace("/")}>
              <Text style={styles.pillText}>Gallery</Text>
            </Pressable>
            <Pressable style={styles.pillDark} onPress={() => setShowing(false)}>
              <Text style={styles.pillText}>Edit</Text>
            </Pressable>
          </View>
        )}
      </View>
    );
  }

  return (
    <ScrollView style={styles.root} contentContainerStyle={styles.content}>
      <Stack.Screen
        options={{
          title: cardId
            ? "Edit card"
            : cardKind === "collectible"
              ? "New collectible"
              : cardKind === "agent"
                ? "New agent"
                : "Make your card",
          headerTitleAlign: "center",
        }}
      />

      {videoUri ? (
        <Video
          source={{ uri: videoUri }}
          style={styles.preview}
          resizeMode={ResizeMode.COVER}
          isLooping
          shouldPlay
          isMuted
        />
      ) : imageUri ? (
        <Image source={{ uri: imageUri }} style={styles.preview} resizeMode="cover" />
      ) : (
        <View style={[styles.preview, styles.previewEmpty]}>
          <Text style={styles.previewHint}>Your photo or AI scene becomes the card front</Text>
        </View>
      )}

      <View style={styles.photoRow}>
        <Pressable style={styles.photoBtn} onPress={takePhoto}>
          <Text style={styles.photoText}>Take photo</Text>
        </Pressable>
        <Pressable style={styles.photoBtn} onPress={pickFromLibrary}>
          <Text style={styles.photoText}>Choose photo</Text>
        </Pressable>
      </View>

      <View style={styles.photoRow}>
        <Pressable style={styles.aiBtn} onPress={() => router.push("/generate-image")}>
          <MaterialCommunityIcons name="auto-fix" size={22} color="#f5f5f7" />
          <Text style={styles.photoText}>AI scene</Text>
        </Pressable>
        <Pressable style={styles.aiBtn} onPress={() => router.push("/generate-video")}>
          <MaterialCommunityIcons name="movie-open-outline" size={22} color="#f5f5f7" />
          <Text style={styles.photoText}>AI video</Text>
        </Pressable>
      </View>

      {cardKind === "agent" && (
        <>
          <Text style={styles.sectionLabel}>Agent details · shown on the flip side</Text>
          <AgentDetailsEditor meta={agentMeta} onChange={setAgentMeta} />
        </>
      )}

      <Pressable
        style={[styles.cta, (!media || saving) && styles.ctaDisabled]}
        disabled={!media || saving}
        onPress={save}
      >
        {saving ? <ActivityIndicator color="#fff" /> : <Text style={styles.ctaText}>Save card</Text>}
      </Pressable>

      {cardId && (
        <Pressable onPress={onDelete}>
          <Text style={styles.deleteText}>Delete card</Text>
        </Pressable>
      )}
    </ScrollView>
  );
}

function Field({
  label,
  ...props
}: { label: string } & React.ComponentProps<typeof TextInput>) {
  return (
    <View style={styles.field}>
      <Text style={styles.label}>{label}</Text>
      <TextInput {...props} style={styles.input} placeholderTextColor="#6b6b70" />
    </View>
  );
}

const styles = StyleSheet.create({
  root: { flex: 1, backgroundColor: "#0a0a0c" },
  content: { padding: 20, gap: 14 },
  h1: { color: "#f5f5f7", fontSize: 28, fontWeight: "800" },
  preview: {
    width: "100%",
    aspectRatio: 2 / 3,
    borderRadius: 20,
    backgroundColor: "#15151a",
    maxHeight: 360,
    alignSelf: "center",
  },
  previewEmpty: { alignItems: "center", justifyContent: "center" },
  previewHint: { color: "#6b6b70" },
  photoRow: { flexDirection: "row", gap: 12 },
  photoBtn: {
    flex: 1,
    backgroundColor: "#222228",
    borderRadius: 14,
    paddingVertical: 14,
    alignItems: "center",
  },
  photoText: { color: "#f5f5f7", fontWeight: "600", fontSize: 16 },
  aiBtn: {
    flex: 1,
    flexDirection: "row",
    gap: 8,
    backgroundColor: "#222228",
    borderRadius: 14,
    paddingVertical: 14,
    alignItems: "center",
    justifyContent: "center",
  },
  field: { gap: 6 },
  label: { color: "#9a9aa0", fontSize: 14 },
  sectionLabel: { color: "#f5f5f7", fontSize: 16, fontWeight: "700", marginTop: 12 },
  input: {
    backgroundColor: "#15151a",
    color: "#f5f5f7",
    borderRadius: 12,
    paddingHorizontal: 14,
    paddingVertical: 12,
    fontSize: 16,
  },
  cta: {
    backgroundColor: "#ff3b30",
    borderRadius: 16,
    paddingVertical: 16,
    alignItems: "center",
    marginTop: 8,
  },
  ctaDisabled: { opacity: 0.4 },
  publishBtn: {
    flexDirection: "row",
    gap: 8,
    backgroundColor: "#222228",
    borderRadius: 16,
    paddingVertical: 15,
    alignItems: "center",
    justifyContent: "center",
  },
  ctaText: { color: "#fff", fontWeight: "700", fontSize: 17 },
  deleteText: { color: "#ff453a", textAlign: "center", paddingVertical: 14, fontWeight: "600" },
  viewerRoot: { flex: 1, backgroundColor: "#0a0a0c" },
  swipeHint: {
    position: "absolute",
    bottom: 96,
    alignSelf: "center",
    color: "#9a9aa0",
    fontSize: 14,
  },
  viewerActions: {
    position: "absolute",
    bottom: 36,
    alignSelf: "center",
    flexDirection: "row",
    gap: 12,
  },
  pillDark: {
    backgroundColor: "#222228",
    borderRadius: 20,
    paddingHorizontal: 24,
    paddingVertical: 12,
  },
  pillText: { color: "#f5f5f7", fontWeight: "600" },
});
