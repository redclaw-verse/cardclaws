// On-device card gallery for the standalone demo (no login/backend). Cards are
// persisted to encrypted MMKV so they survive app restarts; the photo itself is
// copied into permanent storage (see utils/imageStore).

import { MMKV } from "react-native-mmkv";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

const storage = new MMKV({ id: "cardclaws-local-cards" });

export type LinkKind = "domain" | "github" | "publication" | "link";

/** A scannable link shown on the back of the card. */
export interface CardLink {
  id: string;
  kind: LinkKind;
  label: string;
  url: string;
}

export interface LocalCard {
  id: string;
  name: string;
  title: string;
  url: string;
  /** Persistent file:// path to the card photo. */
  imagePath: string;
  /** Persistent file:// path to an AI video for the card front (optional). */
  videoPath?: string;
  /** Back-of-card links (domain, repos, publications, services…). */
  links: CardLink[];
  /** Public profile URL once published to the web (cardclaws.com/<handle>). */
  publishedUrl?: string;
  updatedAt: number;
}

interface LocalCardsState {
  cards: LocalCard[];
  upsert: (card: LocalCard) => void;
  remove: (id: string) => void;
  clear: () => void;
  getById: (id: string) => LocalCard | undefined;
}

export const useLocalCardsStore = create<LocalCardsState>()(
  persist(
    (set, get) => ({
      cards: [],
      upsert: (card) =>
        set((s) => ({
          cards: [card, ...s.cards.filter((c) => c.id !== card.id)].sort(
            (a, b) => b.updatedAt - a.updatedAt,
          ),
        })),
      remove: (id) => set((s) => ({ cards: s.cards.filter((c) => c.id !== id) })),
      clear: () => set({ cards: [] }),
      getById: (id) => get().cards.find((c) => c.id === id),
    }),
    {
      name: "local-cards",
      storage: createJSONStorage(() => ({
        getItem: (k) => storage.getString(k) ?? null,
        setItem: (k, v) => storage.set(k, v),
        removeItem: (k) => storage.delete(k),
      })),
    },
  ),
);
