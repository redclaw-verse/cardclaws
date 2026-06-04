// Builder state with undo/redo (PRD §8.3, §15.2 `cardStore.test.ts`).
//
// Mutations snapshot the card definition before applying, pushing onto a capped
// history stack. This is the memento form of the PRD's command pattern — every
// edit is reversible and the 50-step stack is bounded. Implemented on a vanilla
// Zustand store so it is usable both in React and in headless unit tests.

import { create } from "zustand";
import { BackgroundConfig, CardDefinition, Layer } from "../types/card";

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
  setBackground: (side: Side, background: BackgroundConfig) => void;

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

    setBackground: (side, background) =>
      mutate((card) => {
        card[side] = { ...card[side], background };
        return card;
      }),

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
