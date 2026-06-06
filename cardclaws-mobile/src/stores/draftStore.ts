// Tiny in-memory hand-off: the AI wizard writes the generated image here, and
// the card editor picks it up when it regains focus.

import { create } from "zustand";
import type { AgentBrain } from "../api/brainhub";

interface DraftState {
  pendingImageUri: string | null;
  setPendingImageUri: (uri: string | null) => void;
  pendingVideoUri: string | null;
  setPendingVideoUri: (uri: string | null) => void;
  /** Brain pulled from ClawBrainHub, awaiting the agent-card editor. */
  pendingAgentBrain: AgentBrain | null;
  setPendingAgentBrain: (b: AgentBrain | null) => void;
}

export const useDraftStore = create<DraftState>((set) => ({
  pendingImageUri: null,
  setPendingImageUri: (uri) => set({ pendingImageUri: uri }),
  pendingVideoUri: null,
  setPendingVideoUri: (uri) => set({ pendingVideoUri: uri }),
  pendingAgentBrain: null,
  setPendingAgentBrain: (b) => set({ pendingAgentBrain: b }),
}));
