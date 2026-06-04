// Axios instance with auth + refresh interceptors (PRD §8.1).

import axios, { AxiosError, InternalAxiosRequestConfig } from "axios";
import { useAuthStore } from "../stores/authStore";

export const API_BASE = process.env.EXPO_PUBLIC_API_BASE ?? "http://localhost:8080";
/** Public profile base, e.g. `https://cardclaws.com` — used to build QR/profile URLs. */
export const PROFILE_BASE = process.env.EXPO_PUBLIC_PROFILE_BASE ?? "https://cardclaws.com";

export const api = axios.create({ baseURL: API_BASE });

api.interceptors.request.use((config) => {
  const token = useAuthStore.getState().accessToken;
  if (token) config.headers.Authorization = `Bearer ${token}`;
  return config;
});

// On a 401, attempt a single refresh-token rotation and replay the request.
let refreshing: Promise<boolean> | null = null;

api.interceptors.response.use(
  (res) => res,
  async (error: AxiosError) => {
    const original = error.config as InternalAxiosRequestConfig & { _retried?: boolean };
    if (error.response?.status === 401 && original && !original._retried) {
      original._retried = true;
      refreshing ??= useAuthStore.getState().refresh();
      const ok = await refreshing;
      refreshing = null;
      if (ok) {
        const token = useAuthStore.getState().accessToken;
        if (token) original.headers.Authorization = `Bearer ${token}`;
        return api(original);
      }
    }
    return Promise.reject(error);
  },
);
