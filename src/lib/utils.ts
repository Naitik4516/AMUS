import { open, BaseDirectory } from "@tauri-apps/plugin-fs";

export async function getCoverUrl(path: string) {
  const file = await open("covers/" + path, {
    read: true,
    baseDir: BaseDirectory.AppData,
  });
  const stat = await file.stat();
  const buf = new Uint8Array(stat.size);
  await file.read(buf);
  const blob = new Blob([buf], { type: "image/*" });
  return URL.createObjectURL(blob);
}

export async function getArtistPicUrl(path: string) {
  const file = await open("artists/" + path, {
    read: true,
    baseDir: BaseDirectory.AppData,
  });
  const stat = await file.stat();
  const buf = new Uint8Array(stat.size);
  await file.read(buf);
  const blob = new Blob([buf], { type: "image/*" });
  return URL.createObjectURL(blob);
}

export function formatDuration(s: number, inWords = false) {
  const hours = Math.floor(s / 3600);
  const minutes = Math.floor((s % 3600) / 60);
  const seconds = s % 60;

  if (inWords) {
    const parts = [];
    if (hours > 0) parts.push(`${hours} hour${hours > 1 ? "s" : ""}`);
    if (minutes > 0) parts.push(`${minutes} minute${minutes > 1 ? "s" : ""}`);
    if (seconds > 0) parts.push(`${seconds} second${seconds > 1 ? "s" : ""}`);
    return parts.join(" ");
  } else {
    if (hours > 0)
      return `${hours}:${minutes.toString().padStart(2, "0")}:${seconds.toString().padStart(2, "0")}`;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  }
}
