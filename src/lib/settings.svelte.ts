import { load } from "@tauri-apps/plugin-store";

interface SettingsMap {
  realtimeSync: boolean;
  syncOnStartup: boolean;
  autoFetchArtistPic: boolean;
  autoFetchArtistBanner: boolean;
  keepRunningInBg: boolean;
  autoCheckUpdates: boolean;
  autoplayEnabled: boolean;
  osMediaControls: boolean;
  smoothScrollEnabled: boolean;
  smoothScrollLerp: number;
  smoothScrollDuration: number;
}

type SettingsKey = keyof SettingsMap;
type SettingsValue = SettingsMap[SettingsKey];

const DEFAULTS: SettingsMap = {
  realtimeSync: true,
  syncOnStartup: true,
  autoFetchArtistPic: true,
  autoFetchArtistBanner: true,
  keepRunningInBg: true,
  autoCheckUpdates: true,
  autoplayEnabled: true,
  osMediaControls: true,
  smoothScrollEnabled: true,
  smoothScrollLerp: 0.1,
  smoothScrollDuration: 1.2,
};

let store: Awaited<ReturnType<typeof load>> | null = null;
let initialized = false;

export const settings = $state<SettingsMap>({ ...DEFAULTS });
export const flags = $state({ ready: false });

export async function initSettings(): Promise<void> {
  if (initialized) return;
  initialized = true;
  store = await load("settings.json", { autoSave: true, defaults: {} });
  for (const key of Object.keys(DEFAULTS) as SettingsKey[]) {
    const val = await store.get(key);
    if (val !== undefined) {
      (settings as unknown as Record<string, SettingsValue>)[key] = val as SettingsValue;
    }
  }
  flags.ready = true;
}

export async function setSetting<K extends SettingsKey>(
  key: K,
  value: SettingsMap[K],
): Promise<void> {
  (settings as unknown as Record<string, SettingsValue>)[key] = value;
  const s = store ?? (await load("settings.json", { autoSave: true, defaults: {} }));
  if (!store) store = s;
  await s.set(key, value);
  await s.save();
}

export async function getSetting<K extends SettingsKey>(
  key: K,
  defaultVal: SettingsMap[K],
): Promise<SettingsMap[K]> {
  const s = store ?? (await load("settings.json", { autoSave: true, defaults: {} }));
  if (!store) store = s;
  const val = await s.get(key);
  return val === undefined ? defaultVal : (val as SettingsMap[K]);
}
