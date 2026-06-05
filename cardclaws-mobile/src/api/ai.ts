// AI generator API (PRD §21 Phase 5). The Gemini key lives server-side; the app
// only talks to our backend proxy.

import { api } from "./client";

export interface SceneBrief {
  scene: string;
  style?: string;
  mood?: string;
}

/** Refine the rough fields into a single vivid prompt (Gemini text). */
export async function refineScene(brief: SceneBrief): Promise<string> {
  const res = await api.post<{ prompt: string }>("/v1/ai/refine", brief, { timeout: 60000 });
  return res.data.prompt;
}

/** Generate the card image from a prompt (nano-banana). Returns base64 + mime. */
export async function generateImage(
  prompt: string,
): Promise<{ mimeType: string; imageBase64: string }> {
  const res = await api.post<{ mimeType: string; imageBase64: string }>(
    "/v1/ai/image",
    { prompt },
    { timeout: 120000 },
  );
  return res.data;
}

/** Submit a Veo video job; returns the long-running operation id. */
export async function startVideo(prompt: string): Promise<string> {
  const res = await api.post<{ operationId: string }>("/v1/ai/video", { prompt }, { timeout: 60000 });
  return res.data.operationId;
}

export interface VideoStatus {
  status: "pending" | "done" | "failed";
  mimeType?: string;
  videoBase64?: string;
  error?: string;
}

/** Poll a Veo operation; returns the clip (base64) once `status === "done"`. */
export async function pollVideo(operationId: string): Promise<VideoStatus> {
  const res = await api.post<VideoStatus>(
    "/v1/ai/video/status",
    { operationId },
    { timeout: 120000 },
  );
  return res.data;
}
