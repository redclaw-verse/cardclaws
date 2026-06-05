// Copies picked/captured photos into the app's permanent document directory so
// saved cards keep their image across restarts (ImagePicker URIs live in cache
// and can be evicted).

import * as FileSystem from "expo-file-system";

const DIR = `${FileSystem.documentDirectory}cards/`;

/** Persist `srcUri` as the image for card `id`; returns the stable file path. */
export async function persistImage(srcUri: string, id: string): Promise<string> {
  const dest = `${DIR}${id}.jpg`;
  if (srcUri === dest) return dest; // already persisted (editing without a new photo)
  await FileSystem.makeDirectoryAsync(DIR, { intermediates: true }).catch(() => {});
  await FileSystem.deleteAsync(dest, { idempotent: true }).catch(() => {});
  await FileSystem.copyAsync({ from: srcUri, to: dest });
  return dest;
}

export async function deleteImage(path: string): Promise<void> {
  await FileSystem.deleteAsync(path, { idempotent: true }).catch(() => {});
}

/** Persist `srcUri` as the AI video for card `id`; returns the stable file path. */
export async function persistVideo(srcUri: string, id: string): Promise<string> {
  const dest = `${DIR}${id}.mp4`;
  if (srcUri === dest) return dest;
  await FileSystem.makeDirectoryAsync(DIR, { intermediates: true }).catch(() => {});
  await FileSystem.deleteAsync(dest, { idempotent: true }).catch(() => {});
  await FileSystem.copyAsync({ from: srcUri, to: dest });
  return dest;
}
