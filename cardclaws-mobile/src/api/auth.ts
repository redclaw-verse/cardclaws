// Auth API calls (PRD §13.1).

import { AuthUser, useAuthStore } from "../stores/authStore";
import { api } from "./client";

interface TokenResponse {
  access_token: string;
  refresh_token: string;
  user: {
    id: string;
    email: string;
    handle: string;
    display_name: string;
    tier: AuthUser["tier"];
  };
}

function commit(res: TokenResponse): void {
  useAuthStore.getState().setSession({
    accessToken: res.access_token,
    refreshToken: res.refresh_token,
    user: {
      id: res.user.id,
      email: res.user.email,
      handle: res.user.handle,
      displayName: res.user.display_name,
      tier: res.user.tier,
    },
  });
}

export async function register(input: {
  email: string;
  password: string;
  handle: string;
  displayName: string;
}): Promise<void> {
  const res = await api.post<TokenResponse>("/v1/auth/register", {
    email: input.email,
    password: input.password,
    handle: input.handle,
    display_name: input.displayName,
  });
  commit(res.data);
}

export async function login(email: string, password: string): Promise<void> {
  const res = await api.post<TokenResponse>("/v1/auth/login", { email, password });
  commit(res.data);
}

export async function logout(): Promise<void> {
  const { refreshToken, clear } = useAuthStore.getState();
  if (refreshToken) {
    await api.post("/v1/auth/logout", { refresh_token: refreshToken }).catch(() => {});
  }
  clear();
}
