// Share-link API (PRD §13.4).

import { api } from "./client";

export type ShareModality =
  | "nfc"
  | "qr"
  | "airdrop"
  | "imessage"
  | "email"
  | "link"
  | "wallet"
  | "contact";

export interface ShareLink {
  token: string;
  url: string;
}

/** Create a tracked share link for a card and return its short URL. */
export async function createShareLink(
  cardId: string,
  modality: ShareModality,
  campaign?: string,
): Promise<ShareLink> {
  const res = await api.post<ShareLink>(`/v1/cards/${cardId}/share`, {
    modality,
    campaign,
  });
  return res.data;
}
