import { load } from "@tauri-apps/plugin-store";

type SettingsKey =
  | "realtimeSync"
  | "syncOnStartup"
  | "autoFetchArtistPic"
  | "autoFetchArtistBanner"
  | "keepRunningInBg"
  | "autoCheckUpdates"
  | "autoplayEnabled"
  | "osMediaControls";

const DEFAULTS: Record<SettingsKey, boolean> = {
  realtimeSync: true,
  syncOnStartup: true,
  autoFetchArtistPic: true,
  autoFetchArtistBanner: true,
  keepRunningInBg: true,
  autoCheckUpdates: true,
  autoplayEnabled: true,
  osMediaControls: true,
};

let store: Awaited<ReturnType<typeof load>> | null = null;
let initialized = false;

export const settings = $state<Record<SettingsKey, boolean>>({ ...DEFAULTS });
export const flags = $state({ ready: false });

export async function initSettings(): Promise<void> {
  if (initialized) return;
  initialized = true;
  store = await load("settings.json", { autoSave: true, defaults: {} });
  for (const key of Object.keys(DEFAULTS) as SettingsKey[]) {
    const val = await store.get(key);
    if (val !== undefined) {
      settings[key] = val as boolean;
    }
  }
  flags.ready = true;
}

export async function setSetting(key: SettingsKey, value: boolean): Promise<void> {
  settings[key] = value;
  const s = store ?? (await load("settings.json", { autoSave: true, defaults: {} }));
  if (!store) store = s;
  await s.set(key, value);
  await s.save();
}

export async function getSetting(key: SettingsKey, defaultVal: boolean): Promise<boolean> {
  const s = store ?? (await load("settings.json", { autoSave: true, defaults: {} }));
  if (!store) store = s;
  const val = await s.get(key);
  return val === undefined ? defaultVal : (val as boolean);
}
