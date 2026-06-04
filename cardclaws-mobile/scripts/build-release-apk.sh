#!/usr/bin/env bash
# Build a standalone, signed release APK (JS bundled in — runs with no Metro,
# no laptop). Reproducible: the release signing config is injected at prebuild
# by plugins/withReleaseSigning.js, which reads credentials/keystore.properties.
#
# Usage:
#   JAVA_HOME=$(/usr/libexec/java_home -v 17) ./scripts/build-release-apk.sh
#
# Override the (demo) keystore password via KEYSTORE_PASSWORD if you like.
set -euo pipefail

cd "$(dirname "$0")/.."
PASS="${KEYSTORE_PASSWORD:-cardclaws-demo}"
KS="$(pwd)/credentials/cardclaws-release.keystore"

# 1) Create the release keystore once (gitignored).
if [ ! -f "$KS" ]; then
  echo "Generating release keystore…"
  mkdir -p credentials
  keytool -genkeypair -v -keystore "$KS" -alias cardclaws \
    -keyalg RSA -keysize 2048 -validity 10000 \
    -storepass "$PASS" -keypass "$PASS" \
    -dname "CN=CardClaws, O=RedClaw Systems LLC, C=US"
  cat > credentials/keystore.properties <<EOF
storeFile=$KS
storePassword=$PASS
keyAlias=cardclaws
keyPassword=$PASS
EOF
fi

# 2) Regenerate native project (applies the signing config plugin).
npx expo prebuild --platform android --clean --no-install

# 3) Build + install the signed release APK to the connected device.
npx expo run:android --variant release

APK="android/app/build/outputs/apk/release/app-release.apk"
if [ -f "$APK" ]; then
  cp "$APK" cardclaws-demo.apk
  echo "Standalone APK: $(pwd)/cardclaws-demo.apk"
fi
