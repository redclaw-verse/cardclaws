// Full-screen card viewer: entry animation, flip, and ambient idle drift
// (PRD §6.2). Ambient brightness boost requires expo-brightness (added with the
// hardware integration milestone); the idle motion is implemented here.

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
import { CardDefinition } from "../../types/card";
import { CardFace } from "./CardFace";
import { CardFlip } from "./CardFlip";

export function CardViewer({ card }: { card: CardDefinition }) {
  const { width, height } = useWindowDimensions();
  const cardWidth = Math.min(width * 0.92, 380);
  const cardHeight = Math.min(height * 0.72, cardWidth * 1.5);

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
      { scale: 0.92 + 0.08 * entry.value },
      { translateY: float.value * 6 },
    ],
  }));

  return (
    <View style={styles.root}>
      <Animated.View style={containerStyle}>
        <CardFlip
          width={cardWidth}
          height={cardHeight}
          durationMs={card.settings.flipDurationMs}
          hapticEnabled={card.settings.hapticEnabled}
          gesture={card.settings.flipGesture}
          front={<CardFace side={card.face} width={cardWidth} height={cardHeight} />}
          back={<CardFace side={card.back} width={cardWidth} height={cardHeight} />}
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
});
