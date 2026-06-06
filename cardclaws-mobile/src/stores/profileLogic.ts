// Pure profile logic + option lists (no MMKV import, so it's unit-testable under
// the node jest preset). The MMKV-bound store lives in profileStore.ts.

import type { CardLink } from "./localCardsStore";

export type Goal = "networking" | "sales" | "creator" | "recruiting";

export interface Profile {
  displayName: string;
  title: string;
  industry: string;
  vibes: string[];
  goal: Goal | "";
  brandColors: string[];
  socialLinks: CardLink[];
  inspirations: string[];
}

export const EMPTY_PROFILE: Profile = {
  displayName: "",
  title: "",
  industry: "",
  vibes: [],
  goal: "",
  brandColors: [],
  socialLinks: [],
  inspirations: [],
};

export const VIBE_OPTIONS = [
  "Cinematic",
  "Minimal",
  "Neon",
  "Luxe",
  "Studio",
  "Nature",
  "Abstract",
  "Playful",
  "Dark",
  "Bold",
];

export const GOAL_OPTIONS: { value: Goal; label: string }[] = [
  { value: "networking", label: "Networking" },
  { value: "sales", label: "Sales" },
  { value: "creator", label: "Creator" },
  { value: "recruiting", label: "Recruiting" },
];

export const INDUSTRY_OPTIONS = [
  "Tech",
  "Design",
  "Finance",
  "Healthcare",
  "Real estate",
  "Marketing",
  "Music",
  "Film",
  "Education",
  "Other",
];

export const BRAND_COLOR_SWATCHES = [
  "#ff3b30",
  "#ff9f0a",
  "#ffd60a",
  "#30d158",
  "#0a84ff",
  "#5e5ce6",
  "#bf5af2",
  "#ff2d55",
  "#f5f5f7",
  "#1c1c22",
];

export const INSPIRATION_OPTIONS = [
  "Apple",
  "Linear",
  "Cyberpunk",
  "Editorial",
  "Brutalist",
  "Vaporwave",
  "Swiss",
  "Y2K",
  "Organic",
  "Noir",
];

/** The persona context passed to the AI to tailor generated assets. */
export interface Persona {
  role?: string;
  vibe?: string;
  colors?: string[];
  goal?: string;
}

export function isProfileComplete(p: Profile): boolean {
  return p.displayName.trim().length > 0;
}

/** Map a profile to the AI persona, dropping empty fields; undefined if empty. */
export function buildPersona(p: Profile): Persona | undefined {
  const persona: Persona = {};
  if (p.title.trim()) persona.role = p.title.trim();
  if (p.vibes.length) persona.vibe = p.vibes.join(", ");
  if (p.brandColors.length) persona.colors = p.brandColors;
  if (p.goal) persona.goal = p.goal;
  return Object.keys(persona).length > 0 ? persona : undefined;
}
