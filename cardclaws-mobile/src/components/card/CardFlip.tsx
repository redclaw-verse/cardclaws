// 3D flip container (PRD §6.2.2, §16.2). A Reanimated shared value drives a
// rotateY from 0°→180° with a custom card-flip bezier; faces use backface
// hiding so only the visible side shows. Tap or horizontal swipe flips.

import { ReactNode, useCallback } from "react";
import { StyleSheet, View } from "react-native";
import { Gesture, GestureDetector } from "react-native-gesture-handler";
import Animated, {
  Easing,
  interpolate,
  runOnJS,
  useAnimatedStyle,
  useSharedValue,
  withTiming,
} from "react-native-reanimated";
import * as Haptics from "expo-haptics";

const FLIP_EASING = Easing.bezier(0.645, 0.045, 0.355, 1);
const PERSPECTIVE = 1200;

interface Props {
  front: ReactNode;
  back: ReactNode;
  width: number;
  height: number;
  durationMs?: number;
  hapticEnabled?: boolean;
  gesture?: "swipe" | "doubleTap" | "both";
}

export function CardFlip({
  front,
  back,
  width,
  height,
  durationMs = 400,
  hapticEnabled = true,
  gesture = "both",
}: Props) {
  // 0 = front, 1 = back.
  const progress = useSharedValue(0);

  const fireHaptic = useCallback(() => {
    if (hapticEnabled) Haptics.impactAsync(Haptics.ImpactFeedbackStyle.Medium);
  }, [hapticEnabled]);

  const toggle = useCallback(() => {
    "worklet";
    const next = progress.value < 0.5 ? 1 : 0;
    progress.value = withTiming(next, { duration: durationMs, easing: FLIP_EASING });
    runOnJS(fireHaptic)();
  }, [durationMs, fireHaptic, progress]);

  const tap = Gesture.Tap().onEnd(toggle);
  const swipe = Gesture.Fling()
    .direction(1 /* RIGHT */ | 2 /* LEFT */)
    .onEnd(toggle);
  const composed =
    gesture === "swipe" ? swipe : gesture === "doubleTap" ? tap : Gesture.Race(tap, swipe);

  const frontStyle = useAnimatedStyle(() => ({
    transform: [
      { perspective: PERSPECTIVE },
      { rotateY: `${interpolate(progress.value, [0, 1], [0, 180])}deg` },
    ],
    backfaceVisibility: "hidden",
  }));

  const backStyle = useAnimatedStyle(() => ({
    transform: [
      { perspective: PERSPECTIVE },
      { rotateY: `${interpolate(progress.value, [0, 1], [180, 360])}deg` },
    ],
    backfaceVisibility: "hidden",
  }));

  return (
    <GestureDetector gesture={composed}>
      <View style={{ width, height }}>
        <Animated.View style={[styles.face, frontStyle]}>{front}</Animated.View>
        <Animated.View style={[styles.face, backStyle]}>{back}</Animated.View>
      </View>
    </GestureDetector>
  );
}

const styles = StyleSheet.create({
  face: {
    ...StyleSheet.absoluteFillObject,
  },
});
