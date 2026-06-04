// Builder state with undo/redo (PRD §8.3, §15.2 `cardStore.test.ts`).
//
// Mutations snapshot the card definition before applying, pushing onto a capped
// history stack. This is the memento form of the PRD's command pattern — every
// edit is reversible and the 50-step stack is bounded. Implemented on a vanilla
// Zustand store so it is usable both in React and in headless unit tests.

import { create } from "zustand";
import { BackgroundConfig, CardDefinition, Layer, ProfileData, ProfileLink } from "../types/card";

export const MAX_HISTORY = 50;

export type Side = "face" | "back";

interface CardState {
  card: CardDefinition | null;
  past: CardDefinition[];
  future: CardDefinition[];

  load: (card: CardDefinition) => void;
  addLayer: (side: Side, layer: Layer) => void;
  updateLayer: (side: Side, layerId: string, patch: Partial<Layer>) => void;
  removeLayer: (side: Side, layerId: string) => void;
  reorderLayer: (side: Side, layerId: string, direction: "up" | "down") => void;
  setBackground: (side: Side, background: BackgroundConfig) => void;

  setBio: (bio: string) => void;
  addLink: (link: ProfileLink) => void;
  updateLink: (linkId: string, patch: Partial<ProfileLink>) => void;
  removeLink: (linkId: string) => void;

  undo: () => void;
  redo: () => void;
  canUndo: () => boolean;
  canRedo: () => boolean;
}

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

export function newLayerId(): string {
  const c = (globalThis as { crypto?: { randomUUID?: () => string } }).crypto;
  if (c?.randomUUID) return c.randomUUID();
  return `layer-${Math.floor(Math.random() * 1e9).toString(36)}`;
}

export const useCardStore = create<CardState>((set, get) => {
  /** Apply a mutation while recording the pre-state for undo. */
  const mutate = (fn: (card: CardDefinition) => CardDefinition) => {
    const { card, past } = get();
    if (!card) return;
    const nextPast = [...past, clone(card)];
    if (nextPast.length > MAX_HISTORY) nextPast.shift();
    set({ card: fn(clone(card)), past: nextPast, future: [] });
  };

  const editSide = (
    card: CardDefinition,
    side: Side,
    fn: (layers: Layer[]) => Layer[],
  ): CardDefinition => {
    card[side] = { ...card[side], layers: fn(card[side].layers) };
    return card;
  };

  const emptyProfile = (): ProfileData => ({ bio: "", links: [] });

  const editProfile = (
    card: CardDefinition,
    fn: (profile: ProfileData) => ProfileData,
  ): CardDefinition => {
    card.profile = fn(card.profile ?? emptyProfile());
    return card;
  };

  return {
    card: null,
    past: [],
    future: [],

    load: (card) => set({ card: clone(card), past: [], future: [] }),

    addLayer: (side, layer) =>
      mutate((card) => editSide(card, side, (layers) => [...layers, layer])),

    updateLayer: (side, layerId, patch) =>
      mutate((card) =>
        editSide(card, side, (layers) =>
          layers.map((l) => (l.id === layerId ? ({ ...l, ...patch } as Layer) : l)),
        ),
      ),

    removeLayer: (side, layerId) =>
      mutate((card) =>
        editSide(card, side, (layers) => layers.filter((l) => l.id !== layerId)),
      ),

    reorderLayer: (side, layerId, direction) =>
      mutate((card) =>
        editSide(card, side, (layers) => {
          const sorted = [...layers].sort((a, b) => a.zIndex - b.zIndex);
          const i = sorted.findIndex((l) => l.id === layerId);
          const j = direction === "up" ? i + 1 : i - 1;
          if (i === -1 || j < 0 || j >= sorted.length) return layers;
          // Swap the z-indexes of the two adjacent layers.
          const zi = sorted[i].zIndex;
          sorted[i] = { ...sorted[i], zIndex: sorted[j].zIndex };
          sorted[j] = { ...sorted[j], zIndex: zi };
          return sorted;
        }),
      ),

    setBackground: (side, background) =>
      mutate((card) => {
        card[side] = { ...card[side], background };
        return card;
      }),

    setBio: (bio) => mutate((card) => editProfile(card, (p) => ({ ...p, bio }))),

    addLink: (link) =>
      mutate((card) => editProfile(card, (p) => ({ ...p, links: [...p.links, link] }))),

    updateLink: (linkId, patch) =>
      mutate((card) =>
        editProfile(card, (p) => ({
          ...p,
          links: p.links.map((l) => (l.id === linkId ? { ...l, ...patch } : l)),
        })),
      ),

    removeLink: (linkId) =>
      mutate((card) =>
        editProfile(card, (p) => ({ ...p, links: p.links.filter((l) => l.id !== linkId) })),
      ),

    undo: () => {
      const { card, past, future } = get();
      if (!card || past.length === 0) return;
      const previous = past[past.length - 1];
      set({
        card: previous,
        past: past.slice(0, -1),
        future: [...future, clone(card)],
      });
    },

    redo: () => {
      const { card, past, future } = get();
      if (!card || future.length === 0) return;
      const next = future[future.length - 1];
      set({
        card: next,
        past: [...past, clone(card)],
        future: future.slice(0, -1),
      });
    },

    canUndo: () => get().past.length > 0,
    canRedo: () => get().future.length > 0,
  };
});
