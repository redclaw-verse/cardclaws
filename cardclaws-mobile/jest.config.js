// Two test surfaces:
//  - Pure-logic unit suites (__tests__/unit) run under ts-jest with no React
//    Native dependency, so they pass in CI without a simulator (PRD §15.2 logic
//    suites: handle validation, colour quantiser, vCard, undo/redo store, QR).
//  - Component suites (__tests__/component) would use the `jest-expo` preset and
//    a simulator-class environment; they are authored but not part of the
//    headless unit run.
module.exports = {
  preset: "ts-jest",
  testEnvironment: "node",
  roots: ["<rootDir>/__tests__/unit", "<rootDir>/src"],
  testMatch: ["**/__tests__/unit/**/*.test.ts"],
  transform: {
    "^.+\\.ts$": ["ts-jest", { tsconfig: "tsconfig.spec.json" }],
  },
};
