// Card API calls (PRD §13.2).

import { CardDefinition } from "../types/card";
import { api } from "./client";

export interface CardRecord {
  id: string;
  ownerId: string;
  handle: string;
  status: "draft" | "active" | "archived";
  definition: CardDefinition;
  version: number;
  createdAt: string;
  updatedAt: string;
}

export async function listCards(): Promise<CardRecord[]> {
  const res = await api.get<CardRecord[]>("/v1/cards");
  return res.data;
}

export async function getCard(id: string): Promise<CardRecord> {
  const res = await api.get<CardRecord>(`/v1/cards/${id}`);
  return res.data;
}

export async function createCard(handle: string, definition: CardDefinition): Promise<CardRecord> {
  const res = await api.post<CardRecord>("/v1/cards", { handle, definition });
  return res.data;
}

export async function saveCard(id: string, definition: CardDefinition): Promise<CardRecord> {
  const res = await api.put<CardRecord>(`/v1/cards/${id}`, { definition });
  return res.data;
}

export async function publishCard(id: string): Promise<CardRecord> {
  const res = await api.post<CardRecord>(`/v1/cards/${id}/publish`);
  return res.data;
}

export function appleWalletUrl(id: string): string {
  return `${api.defaults.baseURL}/v1/cards/${id}/wallet/apple`;
}
