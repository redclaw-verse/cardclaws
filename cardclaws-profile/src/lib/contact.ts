// Client-safe contact extraction + vCard 3.0 generation. Mirrors the Rust
// `vcard` module so the web profile produces identical .vcf output without an
// authenticated API call (PRD §6.6.2, §17.3).

import type { CardDefinition } from "./api";

export interface ContactInfo {
  phone?: string;
  email?: string;
  website?: string;
  linkedin?: string;
  company?: string;
  title?: string;
}

/** Find the first `contact` layer's fields across face/back. */
export function extractContact(def: CardDefinition): ContactInfo {
  for (const side of [def.face, def.back]) {
    for (const layer of side?.layers ?? []) {
      if (layer.type === "contact" && layer.fields) {
        return layer.fields as ContactInfo;
      }
    }
  }
  return {};
}

/** Background hex of the face (solid only), defaulting to the dark base. */
export function faceBackground(def: CardDefinition): string {
  const bg = def.face?.background;
  if (bg?.type === "solid" && typeof bg.value === "string") return bg.value;
  return "#101014";
}

function escape(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/;/g, "\\;")
    .replace(/,/g, "\\,")
    .replace(/\n/g, "\\n");
}

export function buildVcard(displayName: string, c: ContactInfo): string {
  const [first, ...rest] = displayName.split(" ");
  const last = rest.join(" ");
  const lines = [
    "BEGIN:VCARD",
    "VERSION:3.0",
    `FN:${escape(displayName)}`,
    `N:${escape(last)};${escape(first ?? "")};;;`,
  ];
  if (c.company) lines.push(`ORG:${escape(c.company)}`);
  if (c.title) lines.push(`TITLE:${escape(c.title)}`);
  if (c.phone) lines.push(`TEL;TYPE=CELL:${escape(c.phone)}`);
  if (c.email) lines.push(`EMAIL:${escape(c.email)}`);
  if (c.website) lines.push(`URL:${escape(c.website)}`);
  if (c.linkedin) lines.push(`X-SOCIALPROFILE;TYPE=linkedin:${escape(c.linkedin)}`);
  lines.push("END:VCARD");
  return lines.join("\r\n") + "\r\n";
}
