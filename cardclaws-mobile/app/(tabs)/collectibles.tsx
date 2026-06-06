// Collectibles: AI-generated cards minted to the blockchain (provenance mapped
// to the creator). Create flow reuses the card creator with kind=collectible.
import { CardCollection } from "../../src/components/CardCollection";

export default function Collectibles() {
  return (
    <CardCollection
      kind="collectible"
      heading="Collectibles"
      addLabel="+ New collectible"
      emptyTitle="No collectibles yet"
      emptyHint="Create one with AI — it's minted to the chain and mapped to you."
    />
  );
}
