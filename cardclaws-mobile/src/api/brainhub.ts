// ClawBrainHub API (via our backend proxy): list agent brains and pull one,
// normalized into card fields (skills/tools/capabilities).

import { api } from "./client";

export interface BrainSummary {
  owner: string;
  name: string;
  description?: string | null;
  version: string;
  trustScore?: number | null;
  badge?: string | null;
}

export interface AgentBrain {
  owner: string;
  name: string;
  version: string;
  tagline: string;
  skills: string[];
  tools: string[];
  capabilities: string[];
}

export async function listBrains(): Promise<BrainSummary[]> {
  const res = await api.get<BrainSummary[]>("/v1/brainhub/brains", { timeout: 30000 });
  return res.data;
}

export async function pullBrain(
  owner: string,
  name: string,
  version: string,
): Promise<AgentBrain> {
  const q = `owner=${encodeURIComponent(owner)}&name=${encodeURIComponent(name)}&version=${encodeURIComponent(version)}`;
  const res = await api.get<AgentBrain>(`/v1/brainhub/pull?${q}`, { timeout: 30000 });
  return res.data;
}
