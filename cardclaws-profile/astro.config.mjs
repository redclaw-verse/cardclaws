import { defineConfig } from "astro/config";
import node from "@astrojs/node";

// Server-rendered at the edge (PRD §11.2). We use the Node adapter for local
// dev/CI verification; production swaps in @astrojs/cloudflare for Cloudflare
// Pages Functions. The page code is adapter-agnostic.
export default defineConfig({
  output: "server",
  adapter: node({ mode: "standalone" }),
});
