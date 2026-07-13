import { load } from "@tauri-apps/plugin-store";

export interface SessionState {
  current_track_id: number | null;
  context_type: string;
  context_id: number | null;
  context_label: string | null;
  context_position: number;
  user_queue_ids: number[];
  position_sec: number;
  volume: number;
  repeat: string;
  shuffle: boolean;
  saved_at: number;
}

const STORE_FILE = "session.json";
const KEY = "playback_session";

let store: Awaited<ReturnType<typeof load>> | null = null;

async function getStore() {
  if (!store) {
    store = await load(STORE_FILE, { autoSave: true, defaults: {} });
  }
  return store;
}

export async function saveSession(state: SessionState): Promise<void> {
  const s = await getStore();
  await s.set(KEY, state);
  await s.save();
}

export async function loadSession(): Promise<SessionState | null> {
  const s = await getStore();
  const val = await s.get<SessionState>(KEY);
  return val ?? null;
}

export async function hasSession(): Promise<boolean> {
  const s = await getStore();
  const val = await s.get(KEY);
  return val !== undefined && val !== null;
}

export async function clearSession(): Promise<void> {
  const s = await getStore();
  await s.delete(KEY);
  await s.save();
}
