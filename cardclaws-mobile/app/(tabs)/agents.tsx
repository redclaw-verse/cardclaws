// Agents: cards for your AI assistants. Front is an AI-generated portrait; the
// flip side shows skill bars, tools, and special capabilities.
import { CardCollection } from "../../src/components/CardCollection";

export default function Agents() {
  return (
    <CardCollection
      kind="agent"
      heading="Agents"
      addLabel="+ New agent"
      emptyTitle="No agent cards yet"
      emptyHint="Generate a portrait of your assistant; flip it for skills & tools."
    />
  );
}
