// Gallery thumbnail. Photo cards show their image; video cards extract a few
// frames and cycle them like a flipbook so they animate and stand out. Frames
// are cached per video path so re-visiting the gallery is instant.

import { MaterialCommunityIcons } from "@expo/vector-icons";
import * as VideoThumbnails from "expo-video-thumbnails";
import { useEffect, useState } from "react";
import { Image, StyleProp, StyleSheet, View, ViewStyle } from "react-native";
import { LocalCard } from "../stores/localCardsStore";

// Spread across the clip (not consecutive frames) so the flipbook shows motion.
const FRAME_TIMES = [0, 1200, 2400, 3600];
const FRAME_CACHE = new Map<string, string[]>();

export function CardThumb({ item, style }: { item: LocalCard; style: StyleProp<ViewStyle> }) {
  const video = item.videoPath;
  const [frames, setFrames] = useState<string[]>(() => (video && FRAME_CACHE.get(video)) || []);
  const [idx, setIdx] = useState(0);

  // Extract preview frames once per video.
  useEffect(() => {
    let cancelled = false;
    if (!video || FRAME_CACHE.has(video)) return;
    (async () => {
      const out: string[] = [];
      for (const time of FRAME_TIMES) {
        try {
          const { uri } = await VideoThumbnails.getThumbnailAsync(video, { time, quality: 0.6 });
          out.push(uri);
        } catch {
          /* a frame failed to extract — skip it */
        }
      }
      if (out.length) FRAME_CACHE.set(video, out);
      if (!cancelled && out.length) setFrames(out);
    })();
    return () => {
      cancelled = true;
    };
  }, [video]);

  // Advance the flipbook.
  useEffect(() => {
    if (frames.length < 2) return;
    const h = setInterval(() => setIdx((i) => (i + 1) % frames.length), 450);
    return () => clearInterval(h);
  }, [frames.length]);

  const src = video && frames.length ? frames[idx] : item.imagePath || null;

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
