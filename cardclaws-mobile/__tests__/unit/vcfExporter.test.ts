import { buildVcard } from "../../src/engine/renderer/vcfExporter";

describe("vcfExporter (RFC 6350)", () => {
  it("produces a well-formed vCard 3.0", () => {
    const vcf = buildVcard("Omar Sobh", {
      title: "Founder",
      company: "RedClaw",
      email: "omar@redclaw.dev",
      phone: "+15551234567",
      website: "https://redclaw.dev",
    });
    expect(vcf.startsWith("BEGIN:VCARD\r\nVERSION:3.0\r\n")).toBe(true);
    expect(vcf).toContain("FN:Omar Sobh\r\n");
    expect(vcf).toContain("N:Sobh;Omar;;;\r\n");
    expect(vcf).toContain("ORG:RedClaw\r\n");
    expect(vcf).toContain("TITLE:Founder\r\n");
    expect(vcf).toContain("TEL;TYPE=CELL:+15551234567\r\n");
    expect(vcf).toContain("EMAIL:omar@redclaw.dev\r\n");
    expect(vcf.trimEnd().endsWith("END:VCARD")).toBe(true);
  });

  it("omits absent fields", () => {
    const vcf = buildVcard("Solo", {});
    expect(vcf).not.toContain("ORG:");
    expect(vcf).not.toContain("TEL");
    expect(vcf).toContain("FN:Solo\r\n");
  });

  it("escapes special characters", () => {
    const vcf = buildVcard("Solo", { company: "Red;Claw, Inc" });
    expect(vcf).toContain("ORG:Red\\;Claw\\, Inc\r\n");
  });

  it("matches the backend output byte-for-byte for a known input", () => {
    const vcf = buildVcard("Omar Sobh", {
      title: "Founder & CEO",
      company: "RedClaw Systems",
      email: "omar@redclaw.dev",
      phone: "+15551234567",
      website: "https://redclaw.dev",
    });
    const expected =
      "BEGIN:VCARD\r\nVERSION:3.0\r\nFN:Omar Sobh\r\nN:Sobh;Omar;;;\r\n" +
      "ORG:RedClaw Systems\r\nTITLE:Founder & CEO\r\nTEL;TYPE=CELL:+15551234567\r\n" +
      "EMAIL:omar@redclaw.dev\r\nURL:https://redclaw.dev\r\nEND:VCARD\r\n";
    expect(vcf).toBe(expected);
  });
});
