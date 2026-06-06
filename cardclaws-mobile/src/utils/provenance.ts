// Collectible "minting" — records on-chain provenance for a generated asset and
// maps it to the creating user. Stubbed for now (deterministic-looking fake tx);
// the real chain submission swaps in behind this same shape (PRD §21, plan
// Phase 2 ProvenanceClient).

import { Provenance } from "../stores/localCardsStore";

const hex = (n: number) =>
  Array.from({ length: n }, () => Math.floor(Math.random() * 16).toString(16)).join("");

/** Mint a collectible to the (stub) chain, owned by `owner`. */
export function mintProvenance(owner: string): Provenance {
  return {
    tokenId: String(Math.floor(Math.random() * 1_000_000)),
    txHash: `0x${hex(64)}`,
    chain: "Base (stub)",
    owner: owner.trim() || "anonymous",
    mintedAt: Date.now(),
  };
}
