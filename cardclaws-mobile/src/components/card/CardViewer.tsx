// Card viewer: entry animation, flip, and ambient idle drift (PRD §6.2).
// Ambient brightness boost requires expo-brightness (added with the hardware
// integration milestone); the idle motion is implemented here.
//
// Sizing: by default the card is a centered, rounded artifact. In `fullScreen`
// mode it fills the entire device window (PRD §5.2 "Full-bleed always").

import { useEffect } from "react";
import { StyleSheet, useWindowDimensions, View } from "react-native";
import Animated, {
  Easing,
  useAnimatedStyle,
  useSharedValue,
  withDelay,
  withRepeat,
  withSequence,
  withTiming,
} from "react-native-reanimated";
import { PROFILE_BASE } from "../../api/client";
import { CardDefinition } from "../../types/card";
import { CardFace } from "./CardFace";
import { CardFlip } from "./CardFlip";

export function CardViewer({
  card,
  profileUrl: profileUrlOverride,
  fullScreen = false,
}: {
  card: CardDefinition;
  /** Override the QR target; defaults to the card's public profile URL. */
  profileUrl?: string;
  /** Fill the entire device window instead of a centered, rounded card. */
  fullScreen?: boolean;
}) {
  // useWindowDimensions is the live size of the app's drawable area; it updates
  // on rotation/resize. Full-screen cards use it verbatim.
  const { width, height } = useWindowDimensions();
  const cardWidth = fullScreen ? width : Math.min(width * 0.92, 380);
  const cardHeight = fullScreen ? height : Math.min(height * 0.72, cardWidth * 1.5);
  const radius = fullScreen ? 0 : 24;
  const profileUrl = profileUrlOverride ?? `${PROFILE_BASE}/${card.handle}`;

  const entry = useSharedValue(0);
  const float = useSharedValue(0);

  useEffect(() => {
    entry.value = withTiming(1, { duration: 450, easing: Easing.out(Easing.cubic) });
    // Ambient drift kicks in after the configured idle delay.
    float.value = withDelay(
      card.settings.ambientModeDelayMs,
      withRepeat(
        withSequence(
          withTiming(1, { duration: 2200, easing: Easing.inOut(Easing.sin) }),
          withTiming(-1, { duration: 2200, easing: Easing.inOut(Easing.sin) }),
        ),
        -1,
        true,
      ),
    );
  }, [card.settings.ambientModeDelayMs, entry, float]);

  const containerStyle = useAnimatedStyle(() => ({
    opacity: entry.value,
    transform: [
      // Full-bleed cards fade in without the scale-from-center (which would
      // reveal the background at the edges); a tiny ambient drift remains.
      { scale: fullScreen ? 1 : 0.92 + 0.08 * entry.value },
      { translateY: float.value * (fullScreen ? 3 : 6) },
    ],
  }));

  return (
    <View style={[styles.root, fullScreen && styles.rootFull]}>
      <Animated.View style={[containerStyle, fullScreen && StyleSheet.absoluteFill]}>
        <CardFlip
          width={cardWidth}
          height={cardHeight}
          durationMs={card.settings.flipDurationMs}
          hapticEnabled={card.settings.hapticEnabled}
          gesture={card.settings.flipGesture}
          front={
            <CardFace
              side={card.face}
              width={cardWidth}
              height={cardHeight}
              borderRadius={radius}
              profileUrl={profileUrl}
            />
          }
          back={
            <CardFace
              side={card.back}
              width={cardWidth}
              height={cardHeight}
              borderRadius={radius}
              profileUrl={profileUrl}
            />
          }
        />
      </Animated.View>
    </View>
  );
}

const styles = StyleSheet.create({
  root: {
    flex: 1,
    alignItems: "center",
    justifyContent: "center",
    backgroundColor: "#0a0a0c",
  },
  rootFull: { alignItems: "stretch", justifyContent: "flex-start" },
});
