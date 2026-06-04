// Typed client for the public CardClaws API used during SSR.

export interface PublicProfile {
  id: string;
  ownerId: string;
  handle: string;
  status: string;
  definition: CardDefinition;
  version: number;
  createdAt: string;
  updatedAt: string;
  ownerDisplayName: string;
}

// Loose mirror of the Rust CardDefinition — only the parts the profile reads.
export interface CardDefinition {
  face?: CardSide;
  back?: CardSide;
  [key: string]: unknown;
}

export interface CardSide {
  layers?: Layer[];
  background?: { type?: string; value?: string };
}

export interface Layer {
  type?: string;
  fields?: Record<string, string>;
  [key: string]: unknown;
}

// Read at runtime from process.env (Node adapter) so the base URL is
// configurable without a rebuild; `import.meta.env` values are inlined at build
// time and would otherwise be frozen. Default to 127.0.0.1 (not `localhost`,
// which Node's fetch may resolve to an unbound ::1).
function runtimeEnv(key: string): string | undefined {
  const proc = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process;
  return proc?.env?.[key] ?? (import.meta.env as Record<string, string | undefined>)[key];
}

export function apiBase(): string {
  return runtimeEnv("API_BASE") ?? "http://127.0.0.1:8080";
}

/** Public base URL the browser uses (client analytics). Falls back to apiBase. */
export function publicApiBase(): string {
  return runtimeEnv("PUBLIC_API_BASE") ?? apiBase();
}

/**
 * Fetch the public profile for a handle. Forwards the visitor's IP/UA so the
 * API attributes the `profile_visit` to the real client, not the SSR server.
 */
export async function fetchProfile(
  handle: string,
  forward: Headers,
): Promise<PublicProfile | null> {
  const headers: Record<string, string> = {};
  for (const h of ["cf-connecting-ip", "x-forwarded-for", "x-real-ip", "user-agent"]) {
    const v = forward.get(h);
    if (v) headers[h] = v;
  }
  const res = await fetch(
    `${apiBase()}/v1/cards/handle/${encodeURIComponent(handle)}`,
    { headers },
  );
  return res.status === 200 ? ((await res.json()) as PublicProfile) : null;
}
