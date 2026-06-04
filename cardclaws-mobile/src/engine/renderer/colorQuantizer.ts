// On-device palette extraction via k-means (k=8) over an image's pixels
// (PRD §6.1.3). Pure function over an RGBA byte array so it is unit-testable
// without any native image decoding.

import { RGB, rgbToHex } from "../../utils/colorUtils";

export interface QuantizeOptions {
  k?: number;
  maxIterations?: number;
  /** Skip every Nth pixel for speed on large images. */
  sampleStride?: number;
}

/**
 * Extract up to `k` dominant colors from an RGBA pixel buffer (4 bytes/pixel).
 * Returns hex strings ordered by cluster population (most dominant first).
 * Deterministic: centroids are seeded by evenly spaced samples, not randomness.
 */
export function quantize(rgba: Uint8Array | number[], opts: QuantizeOptions = {}): string[] {
  const k = opts.k ?? 8;
  const maxIterations = opts.maxIterations ?? 10;
  const stride = Math.max(1, opts.sampleStride ?? 1);

  const pixels: RGB[] = [];
  for (let i = 0; i + 3 < rgba.length; i += 4 * stride) {
    const a = rgba[i + 3];
    if (a < 16) continue; // skip near-transparent
    pixels.push({ r: rgba[i], g: rgba[i + 1], b: rgba[i + 2] });
  }
  if (pixels.length === 0) return [];

  const realK = Math.min(k, pixels.length);
  // Deterministic seeding: evenly spaced samples across the pixel list.
  let centroids: RGB[] = Array.from({ length: realK }, (_, c) => {
    const idx = Math.floor((c * pixels.length) / realK);
    return { ...pixels[idx] };
  });

  let assignment = new Array<number>(pixels.length).fill(0);

  for (let iter = 0; iter < maxIterations; iter++) {
    let moved = false;
    // Assign each pixel to nearest centroid.
    for (let p = 0; p < pixels.length; p++) {
      let best = 0;
      let bestDist = Infinity;
      for (let c = 0; c < realK; c++) {
        const d = dist2(pixels[p], centroids[c]);
        if (d < bestDist) {
          bestDist = d;
          best = c;
        }
      }
      if (assignment[p] !== best) {
        assignment[p] = best;
        moved = true;
      }
    }
    // Recompute centroids.
    const sums = Array.from({ length: realK }, () => ({ r: 0, g: 0, b: 0, n: 0 }));
    for (let p = 0; p < pixels.length; p++) {
      const s = sums[assignment[p]];
      s.r += pixels[p].r;
      s.g += pixels[p].g;
      s.b += pixels[p].b;
      s.n += 1;
    }
    centroids = sums.map((s, c) =>
      s.n === 0 ? centroids[c] : { r: s.r / s.n, g: s.g / s.n, b: s.b / s.n },
    );
    if (!moved && iter > 0) break;
  }

  // Order clusters by population.
  const counts = new Array<number>(realK).fill(0);
  for (const a of assignment) counts[a] += 1;
  return centroids
    .map((c, i) => ({ hex: rgbToHex(c), n: counts[i] }))
    .filter((x) => x.n > 0)
    .sort((a, b) => b.n - a.n)
    .map((x) => x.hex);
}

function dist2(a: RGB, b: RGB): number {
  const dr = a.r - b.r;
  const dg = a.g - b.g;
  const db = a.b - b.b;
  return dr * dr + dg * dg + db * db;
}
