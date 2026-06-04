/// <reference path="../.astro/types.d.ts" />
/// <reference types="astro/client" />

interface ImportMetaEnv {
  /** Base URL of the CardClaws API (server-side fetches). */
  readonly API_BASE?: string;
  /** Base URL exposed to the browser for client-side analytics POSTs. */
  readonly PUBLIC_API_BASE?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
