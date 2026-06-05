// Gallery thumbnail. Photo cards show their image; video cards show a single
// static poster frame (no animation — it flickered) plus a ▶ badge so they're
// still easy to tell apart. Posters are cached per video path.

import { MaterialCommunityIcons } from "@expo/vector-icons";
import * as VideoThumbnails from "expo-video-thumbnails";
import { useEffect, useState } from "react";
import { Image, StyleProp, StyleSheet, View, ViewStyle } from "react-native";
import { LocalCard } from "../stores/localCardsStore";

// A bit into the clip — the very first frame is often black.
const POSTER_TIME = 500;
const POSTER_CACHE = new Map<string, string>();

export function CardThumb({ item, style }: { item: LocalCard; style: StyleProp<ViewStyle> }) {
  const video = item.videoPath;
  const [poster, setPoster] = useState<string | null>(() => (video && POSTER_CACHE.get(video)) || null);

  useEffect(() => {
    let cancelled = false;
    if (!video || POSTER_CACHE.has(video)) return;
    (async () => {
      try {
        const { uri } = await VideoThumbnails.getThumbnailAsync(video, {
          time: POSTER_TIME,
          quality: 0.7,
        });
        POSTER_CACHE.set(video, uri);
        if (!cancelled) setPoster(uri);
      } catch {
        /* couldn't extract — fall back to the placeholder */
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [video]);

  const fallback = item.imagePath || null;
  const src = video ? poster || fallback : fallback;

  return (
    <View style={style}>
      {src ? (
        <Image source={{ uri: src }} style={StyleSheet.absoluteFill} resizeMode="cover" />
      ) : (
        <View style={[StyleSheet.absoluteFill, styles.empty]}>
          <MaterialCommunityIcons name="movie-open" size={30} color="#6b6b70" />
        </View>
      )}
      {!!video && (
        <View style={styles.badge}>
          <MaterialCommunityIcons name="play" size={11} color="#fff" />
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  empty: { backgroundColor: "#1c1c22", alignItems: "center", justifyContent: "center" },
  badge: {
    position: "absolute",
    top: 8,
    right: 8,
    width: 22,
    height: 22,
    borderRadius: 11,
    backgroundColor: "rgba(0,0,0,0.55)",
    alignItems: "center",
    justifyContent: "center",
  },
});
