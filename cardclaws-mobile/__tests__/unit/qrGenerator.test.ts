import { qrMatrix } from "../../src/engine/renderer/qrGenerator";

describe("qrGenerator", () => {
  it("produces a non-empty square matrix", () => {
    const m = qrMatrix("https://cardclaws.com/omar");
    expect(m.length).toBeGreaterThan(0);
    expect(m.every((row) => row.length === m.length)).toBe(true);
  });

  it("encodes a finder pattern in the top-left corner", () => {
    // Every QR code has a 7x7 finder pattern at (0,0): dark border, light ring,
    // dark 3x3 core.
    const m = qrMatrix("hello");
    expect(m[0].slice(0, 7).every((d) => d)).toBe(true); // top row dark
    expect(m[1][0]).toBe(true);
    expect(m[1][1]).toBe(false); // inner light ring
    expect(m[3][3]).toBe(true); // dark core
  });

  it("longer payloads need a larger matrix", () => {
    const small = qrMatrix("a");
    const large = qrMatrix("x".repeat(300));
    expect(large.length).toBeGreaterThan(small.length);
  });
});
