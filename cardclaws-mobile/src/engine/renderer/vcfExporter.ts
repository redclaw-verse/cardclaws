// vCard 3.0 (RFC 6350) serializer — mirrors the backend `vcard` module so the
// app produces identical .vcf output (PRD §15.2 `vcfExporter.test.ts`, §17.3).

import { ContactFields } from "../../types/card";

function escape(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/;/g, "\\;")
    .replace(/,/g, "\\,")
    .replace(/\n/g, "\\n");
}

export function buildVcard(displayName: string, c: ContactFields): string {
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
