// Expo config plugin: inject a release signing config into the generated
// android/app/build.gradle so `assembleRelease` produces a properly-signed,
// standalone APK. Credentials are read at build time from
// credentials/keystore.properties (gitignored) — never baked into source.
//
// This runs during `expo prebuild`, so release signing is reproducible even
// though android/ is regenerated (CNG workflow).

const { withAppBuildGradle } = require("@expo/config-plugins");

const RELEASE_SIGNING_CONFIG = `release {
            def kp = new Properties()
            def kf = rootProject.file("../credentials/keystore.properties")
            if (kf.exists()) {
                kp.load(new FileInputStream(kf))
                storeFile file(kp["storeFile"])
                storePassword kp["storePassword"]
                keyAlias kp["keyAlias"]
                keyPassword kp["keyPassword"]
            }
        }
        debug {`;

module.exports = function withReleaseSigning(config) {
  return withAppBuildGradle(config, (cfg) => {
    let src = cfg.modResults.contents;
    if (src.includes("signingConfigs.release")) return cfg; // already applied

    // 1) Add a `release` signing config alongside `debug`.
    src = src.replace(/signingConfigs \{\s*\n\s*debug \{/, `signingConfigs {\n        ${RELEASE_SIGNING_CONFIG}`);

    // 2) Point the release build type at it (only the release block, anchored on
    //    the shrinkResources line that follows it in the template).
    src = src.replace(
      /signingConfig signingConfigs\.debug(\s*\n\s*shrinkResources)/,
      "signingConfig signingConfigs.release$1",
    );

    cfg.modResults.contents = src;
    return cfg;
  });
};
