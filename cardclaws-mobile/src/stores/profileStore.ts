// The user profile (set during first-run onboarding) — MMKV-persisted, mirrors
// the localCardsStore pattern. `hydrated` flips once persist rehydrates so the
// first-launch gate doesn't flash before MMKV is read.

import { MMKV } from "react-native-mmkv";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";
import { EMPTY_PROFILE, Profile } from "./profileLogic";

const storage = new MMKV({ id: "cardclaws-profile" });

interface ProfileState {
  onboarded: boolean;
  profile: Profile;
  setProfile: (patch: Partial<Profile>) => void;
  completeOnboarding: (p: Profile) => void;
  reset: () => void;
}

// MMKV via createJSONStorage hydrates synchronously, so `onboarded` is correct
// on the first render — no async hydration flag needed (matches localCardsStore).
export const useProfileStore = create<ProfileState>()(
  persist(
    (set) => ({
      onboarded: false,
      profile: EMPTY_PROFILE,
      setProfile: (patch) => set((s) => ({ profile: { ...s.profile, ...patch } })),
      completeOnboarding: (p) => set({ profile: p, onboarded: true }),
      reset: () => set({ profile: EMPTY_PROFILE, onboarded: false }),
    }),
    {
      name: "profile",
      storage: createJSONStorage(() => ({
        getItem: (k) => storage.getString(k) ?? null,
        setItem: (k, v) => storage.set(k, v),
        removeItem: (k) => storage.delete(k),
      })),
      partialize: (s) => ({ onboarded: s.onboarded, profile: s.profile }),
    },
  ),
);
