// Auth session store, persisted to encrypted MMKV (PRD §8.1). Tokens survive
// app restarts; refresh rotates the pair against /v1/auth/refresh.

import axios from "axios";
import { MMKV } from "react-native-mmkv";
import { create } from "zustand";
import { createJSONStorage, persist } from "zustand/middleware";

const API_BASE = process.env.EXPO_PUBLIC_API_BASE ?? "http://localhost:8080";
const storage = new MMKV({ id: "cardclaws-auth", encryptionKey: "cardclaws" });

export interface AuthUser {
  id: string;
  email: string;
  handle: string;
  displayName: string;
  tier: "free" | "pro" | "team" | "enterprise";
}

interface AuthState {
  accessToken: string | null;
  refreshToken: string | null;
  user: AuthUser | null;
  isAuthenticated: () => boolean;
  setSession: (tokens: { accessToken: string; refreshToken: string; user: AuthUser }) => void;
  clear: () => void;
  refresh: () => Promise<boolean>;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      accessToken: null,
      refreshToken: null,
      user: null,
      isAuthenticated: () => get().accessToken !== null,
      setSession: ({ accessToken, refreshToken, user }) =>
        set({ accessToken, refreshToken, user }),
      clear: () => set({ accessToken: null, refreshToken: null, user: null }),
      refresh: async () => {
        const token = get().refreshToken;
        if (!token) return false;
        try {
          const res = await axios.post(`${API_BASE}/v1/auth/refresh`, {
            refresh_token: token,
          });
          set({
            accessToken: res.data.access_token,
            refreshToken: res.data.refresh_token,
          });
          return true;
        } catch {
          get().clear();
          return false;
        }
      },
    }),
    {
      name: "cardclaws-auth",
      storage: createJSONStorage(() => ({
        getItem: (k) => storage.getString(k) ?? null,
        setItem: (k, v) => storage.set(k, v),
        removeItem: (k) => storage.delete(k),
      })),
    },
  ),
);
