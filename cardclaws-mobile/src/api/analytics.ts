// Analytics API (PRD §13.5).

import { api } from "./client";

export interface AnalyticsSummary {
  totalVisits: number;
  visits7d: number;
  visits24h: number;
  qrScans: number;
  contactSaves: number;
  linkClicks: number;
}

export interface FeedEvent {
  id: number;
  eventType: string;
  shareToken: string | null;
  country: string | null;
  city: string | null;
  occurredAt: string;
}

export interface GeoCount {
  country: string;
  visits: number;
}

export async function getSummary(cardId: string): Promise<AnalyticsSummary> {
  return (await api.get<AnalyticsSummary>(`/v1/cards/${cardId}/analytics`)).data;
}

export async function getFeed(cardId: string): Promise<FeedEvent[]> {
  return (await api.get<FeedEvent[]>(`/v1/cards/${cardId}/analytics/feed`)).data;
}

export async function getGeo(cardId: string): Promise<GeoCount[]> {
  return (await api.get<GeoCount[]>(`/v1/cards/${cardId}/analytics/geo`)).data;
}
