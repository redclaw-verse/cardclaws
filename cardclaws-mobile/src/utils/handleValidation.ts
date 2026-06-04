// Handle validation — mirrors the backend rules (cardclaws-api validation.rs)
// so client and server agree (PRD §6.6.1, §6.8.3).

const RESERVED = new Set([
  "admin", "support", "cardclaws", "help", "api", "www", "app", "about",
  "login", "logout", "register", "settings", "profile", "s", "v1", "assets",
]);

export type HandleError =
  | "too_short"
  | "too_long"
  | "leading_or_trailing_hyphen"
  | "invalid_chars"
  | "reserved";

export function validateHandle(handle: string): HandleError | null {
  const len = [...handle].length;
  if (len < 3) return "too_short";
  if (len > 30) return "too_long";
  if (handle.startsWith("-") || handle.endsWith("-")) {
    return "leading_or_trailing_hyphen";
  }
  if (!/^[a-z0-9-]+$/.test(handle)) return "invalid_chars";
  if (RESERVED.has(handle)) return "reserved";
  return null;
}

export function isValidHandle(handle: string): boolean {
  return validateHandle(handle) === null;
}
