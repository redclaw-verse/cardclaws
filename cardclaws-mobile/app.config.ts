import { ExpoConfig } from "expo/config";

// Expo app configuration (PRD §8). iOS-first for Phase 1.
const config: ExpoConfig = {
  name: "CardClaws",
  slug: "cardclaws",
  scheme: "cardclaws",
  version: "0.1.0",
  orientation: "portrait",
  userInterfaceStyle: "automatic",
  ios: {
    bundleIdentifier: "com.cardclaws.app",
    supportsTablet: false,
  },
  android: {
    package: "com.cardclaws.app",
  },
  plugins: [
    "expo-router",
    // Allow cleartext HTTP so the demo can reach the backend over `adb reverse`
    // (localhost:8080). Production scopes this to specific domains.
    ["expo-build-properties", { android: { usesCleartextTraffic: true } }],
    [
      "expo-image-picker",
      {
        photosPermission: "CardClaws uses your photos to set your card image.",
        cameraPermission: "CardClaws uses the camera to capture your card image.",
      },
    ],
    "./plugins/withReleaseSigning.js",
  ],
  experiments: {
    typedRoutes: true,
  },
  extra: {
    apiBase: process.env.EXPO_PUBLIC_API_BASE ?? "http://localhost:8080",
  },
};

export default config;
