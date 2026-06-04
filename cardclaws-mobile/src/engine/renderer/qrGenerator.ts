// On-device QR generation (PRD §15.2 `qrGenerator.test.ts`). Wraps the `qrcode`
// library, exposing the raw module matrix (for Skia rendering on the card back)
// and a data URL (for quick previews / share overlays).

import QRCode from "qrcode";

/**
 * Produce the QR module matrix for `text`. `matrix[row][col] === true` means a
 * dark module. The matrix is always square.
 */
export function qrMatrix(text: string): boolean[][] {
  const qr = QRCode.create(text, { errorCorrectionLevel: "M" });
  const size = qr.modules.size;
  const data = qr.modules.data;
  const matrix: boolean[][] = [];
  for (let row = 0; row < size; row++) {
    const cols: boolean[] = [];
    for (let col = 0; col < size; col++) {
      cols.push(Boolean(data[row * size + col]));
    }
    matrix.push(cols);
  }
  return matrix;
}

export function qrDataUrl(text: string): Promise<string> {
  return QRCode.toDataURL(text, { errorCorrectionLevel: "M", margin: 1, width: 512 });
}
