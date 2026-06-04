import { quantize } from "../../src/engine/renderer/colorQuantizer";

/** Build an RGBA buffer from a list of [r,g,b] colors, each repeated `count`x. */
function buffer(colors: Array<[number, number, number, number]>): number[] {
  const out: number[] = [];
  for (const [r, g, b, count] of colors) {
    for (let i = 0; i < count; i++) out.push(r, g, b, 255);
  }
  return out;
}

describe("colorQuantizer (k-means)", () => {
  it("recovers the dominant colors from a two-color image", () => {
    // 70 red pixels, 30 blue pixels.
    const rgba = buffer([
      [255, 0, 0, 70],
      [0, 0, 255, 30],
    ]);
    const palette = quantize(rgba, { k: 2 });
    expect(palette.length).toBe(2);
    // Most dominant cluster first → red.
    expect(palette[0]).toBe("#ff0000");
    expect(palette).toContain("#0000ff");
  });

  it("skips fully transparent pixels", () => {
    const rgba = [
      255, 255, 255, 0, // transparent white — ignored
      0, 128, 0, 255, // opaque green
      0, 128, 0, 255,
    ];
    const palette = quantize(rgba, { k: 4 });
    expect(palette).toEqual(["#008000"]);
  });

  it("returns empty for an empty buffer", () => {
    expect(quantize([], { k: 8 })).toEqual([]);
  });

  it("is deterministic across runs", () => {
    const rgba = buffer([
      [10, 20, 30, 40],
      [200, 100, 50, 25],
      [5, 250, 120, 35],
    ]);
    expect(quantize(rgba, { k: 3 })).toEqual(quantize(rgba, { k: 3 }));
  });
});
