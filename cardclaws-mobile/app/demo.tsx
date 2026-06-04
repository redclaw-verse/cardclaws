// Standalone, no-login demo (no backend needed): take/pick a photo → it's your
// card front; swipe → the back reveals a QR code + your info. Everything is
// built in-memory on the device.

import * as ImagePicker from "expo-image-picker";
import { Stack } from "expo-router";
import { StatusBar } from "expo-status-bar";
import { useState } from "react";
import {
  Image,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from "react-native";
import { CardViewer } from "../src/components/card/CardViewer";
import { newLayerId } from "../src/stores/cardStore";
import { CardDefinition, DEFAULT_SETTINGS, TextLayer } from "../src/types/card";

function textLayer(text: string, y: number, size: number, color: string): TextLayer {
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
    align: "left",
  };
}

/// Build the demo card in memory: photo as the full-bleed face, QR + info on the
/// back.
function buildDemoCard(imageUri: string, name: string, title: string): CardDefinition {
  return {
    id: "demo",
    ownerId: "demo",
    handle: "demo",
    version: 1,
    face: {
      layers: [
        textLayer(name, 0.78, 30, "#ffffff"),
        title ? textLayer(title, 0.87, 18, "#f0f0f2") : textLayer("", 0.87, 1, "#ffffff"),
      ],
      background: { type: "image", value: imageUri },
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

export default function DemoScreen() {
  const [imageUri, setImageUri] = useState<string | null>(null);
  const [name, setName] = useState("Omar Sobh");
  const [title, setTitle] = useState("Founder & CEO");
  const [url, setUrl] = useState("https://cardclaws.com/omar");
  const [showing, setShowing] = useState(false);

  const pickFromLibrary = async () => {
    const res = await ImagePicker.launchImageLibraryAsync({
      mediaTypes: ImagePicker.MediaTypeOptions.Images,
      allowsEditing: true,
      aspect: [2, 3],
      quality: 0.9,
    });
    if (!res.canceled) setImageUri(res.assets[0].uri);
  };

  const takePhoto = async () => {
    const perm = await ImagePicker.requestCameraPermissionsAsync();
    if (!perm.granted) return;
    const res = await ImagePicker.launchCameraAsync({
      allowsEditing: true,
      aspect: [2, 3],
      quality: 0.9,
    });
    if (!res.canceled) setImageUri(res.assets[0].uri);
  };

  if (showing && imageUri) {
    const card = buildDemoCard(imageUri, name, title);
    return (
      <View style={styles.viewerRoot}>
        {/* Hide the header + status bar so the card is truly full-bleed. */}
        <Stack.Screen options={{ headerShown: false }} />
        <StatusBar hidden />
        <CardViewer card={card} profileUrl={url} fullScreen />
        <Text style={styles.swipeHint}>Swipe or tap the card to flip →</Text>
        <Pressable style={styles.editBtn} onPress={() => setShowing(false)}>
          <Text style={styles.editText}>Edit</Text>
        </Pressable>
      </View>
    );
  }

  return (
    <ScrollView style={styles.root} contentContainerStyle={styles.content}>
      <Stack.Screen options={{ title: "Card demo" }} />
      <Text style={styles.h1}>Make your card</Text>

      {imageUri ? (
        <Image source={{ uri: imageUri }} style={styles.preview} resizeMode="cover" />
      ) : (
        <View style={[styles.preview, styles.previewEmpty]}>
          <Text style={styles.previewHint}>Your photo becomes the card front</Text>
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

      <Field label="Name" value={name} onChangeText={setName} />
      <Field label="Title" value={title} onChangeText={setTitle} />
      <Field label="QR link" value={url} onChangeText={setUrl} autoCapitalize="none" />

      <Pressable
        style={[styles.cta, !imageUri && styles.ctaDisabled]}
        disabled={!imageUri}
        onPress={() => setShowing(true)}
      >
        <Text style={styles.ctaText}>Show my card</Text>
      </Pressable>
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
  field: { gap: 6 },
  label: { color: "#9a9aa0", fontSize: 14 },
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
  ctaText: { color: "#fff", fontWeight: "700", fontSize: 17 },
  viewerRoot: { flex: 1, backgroundColor: "#0a0a0c" },
  swipeHint: {
    position: "absolute",
    bottom: 90,
    alignSelf: "center",
    color: "#9a9aa0",
    fontSize: 14,
  },
  editBtn: {
    position: "absolute",
    bottom: 36,
    alignSelf: "center",
    backgroundColor: "#222228",
    borderRadius: 20,
    paddingHorizontal: 24,
    paddingVertical: 12,
  },
  editText: { color: "#f5f5f7", fontWeight: "600" },
});
