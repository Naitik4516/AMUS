import { load } from "@tauri-apps/plugin-store";

export type ShortcutCategory =
  | "playback"
  | "seeking"
  | "volume"
  | "library"
  | "queue"
  | "navigation"
  | "window";

export interface ShortcutBinding {
  key: string;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
  meta?: boolean;
}

export interface ShortcutAction {
  id: string;
  label: string;
  category: ShortcutCategory;
  defaultKeys: ShortcutBinding[];
  enabled: boolean;
  customKeys?: ShortcutBinding[];
  description?: string;
}

const KEY_DISPLAY: Record<string, string> = {
  Space: "Space",
  ArrowLeft: "←",
  ArrowRight: "→",
  ArrowUp: "↑",
  ArrowDown: "↓",
  Escape: "Esc",
  Enter: "Enter",
  Home: "Home",
  Delete: "Delete",
  MediaPlayPause: "Media Play",
  MediaTrackNext: "Media Next",
  MediaTrackPrevious: "Media Prev",
  MediaStop: "Media Stop",
};

export const ALL_SHORTCUTS: ShortcutAction[] = [
  // Playback
  {
    id: "play_pause",
    label: "Play / Pause",
    category: "playback",
    defaultKeys: [{ key: "Space" }, { key: "k" }],
    enabled: true,
  },
  {
    id: "next_track",
    label: "Next Track",
    category: "playback",
    defaultKeys: [{ key: "n" }, { key: "ArrowRight", ctrl: true }],
    enabled: true,
  },
  {
    id: "prev_track",
    label: "Previous Track",
    category: "playback",
    defaultKeys: [{ key: "b" }, { key: "ArrowLeft", ctrl: true }],
    enabled: true,
  },
  {
    id: "stop",
    label: "Stop",
    category: "playback",
    defaultKeys: [{ key: "Space", shift: true }],
    enabled: true,
  },
  {
    id: "restart_track",
    label: "Restart Current Track",
    category: "playback",
    defaultKeys: [{ key: "Home" }],
    enabled: true,
  },
  // Seeking
  {
    id: "seek_forward_5",
    label: "Seek +5 sec",
    category: "seeking",
    defaultKeys: [{ key: "ArrowRight" }],
    enabled: true,
  },
  {
    id: "seek_backward_5",
    label: "Seek -5 sec",
    category: "seeking",
    defaultKeys: [{ key: "ArrowLeft" }],
    enabled: true,
  },
  {
    id: "seek_forward_10",
    label: "Seek +10 sec",
    category: "seeking",
    defaultKeys: [{ key: "ArrowRight", shift: true }],
    enabled: true,
  },
  {
    id: "seek_backward_10",
    label: "Seek -10 sec",
    category: "seeking",
    defaultKeys: [{ key: "ArrowLeft", shift: true }],
    enabled: true,
  },
  // Volume
  {
    id: "volume_up",
    label: "Volume Up",
    category: "volume",
    defaultKeys: [{ key: "=" }, { key: "+" }],
    enabled: true,
  },
  {
    id: "volume_down",
    label: "Volume Down",
    category: "volume",
    defaultKeys: [{ key: "-" }],
    enabled: true,
  },
  {
    id: "mute",
    label: "Mute",
    category: "volume",
    defaultKeys: [{ key: "m" }],
    enabled: true,
  },
  // Library / Search
  {
    id: "focus_search",
    label: "Focus Search",
    category: "library",
    defaultKeys: [{ key: "f", ctrl: true }, { key: "/" }, { key: "s" }],
    enabled: true,
  },
  {
    id: "refresh_page",
    label: "Refresh Page",
    category: "library",
    defaultKeys: [{ key: "F5" }, { key: "r", ctrl: true }],
    enabled: true,
  },
  {
    id: "rescan_music",
    label: "Rescan Music Folder",
    category: "library",
    defaultKeys: [{ key: "r", ctrl: true, shift: true }],
    enabled: true,
  },
  // Queue
  {
    id: "toggle_shuffle",
    label: "Toggle Shuffle",
    category: "queue",
    defaultKeys: [{ key: "r" }],
    enabled: true,
  },
  {
    id: "toggle_repeat",
    label: "Toggle Repeat",
    category: "queue",
    defaultKeys: [{ key: "l" }],
    enabled: true,
  },
  {
    id: "toggle_queue",
    label: "Toggle Queue Window",
    category: "queue",
    defaultKeys: [{ key: "q" }],
    enabled: true,
  },
  {
    id: "clear_queue",
    label: "Clear Queue",
    category: "queue",
    defaultKeys: [{ key: "Delete", ctrl: true, shift: true }],
    enabled: true,
  },
  // Navigation
  {
    id: "navigate_home",
    label: "Home",
    category: "navigation",
    defaultKeys: [{ key: "h" }],
    enabled: true,
  },
  {
    id: "navigate_library",
    label: "Library",
    category: "navigation",
    defaultKeys: [{ key: "0" }],
    enabled: true,
  },
  {
    id: "navigate_playlists",
    label: "Playlists",
    category: "navigation",
    defaultKeys: [{ key: "1" }],
    enabled: true,
  },
  {
    id: "navigate_albums",
    label: "Albums",
    category: "navigation",
    defaultKeys: [{ key: "2" }],
    enabled: true,
  },
  {
    id: "navigate_artists",
    label: "Artists",
    category: "navigation",
    defaultKeys: [{ key: "3" }],
    enabled: true,
  },
  {
    id: "navigate_settings",
    label: "Settings",
    category: "navigation",
    defaultKeys: [{ key: ",", ctrl: true }],
    enabled: true,
  },
  {
    id: "go_back",
    label: "Go Back",
    category: "navigation",
    defaultKeys: [{ key: "ArrowLeft", alt: true }],
    enabled: true,
  },
  {
    id: "go_forward",
    label: "Go Forward",
    category: "navigation",
    defaultKeys: [{ key: "ArrowRight", alt: true }],
    enabled: true,
  },
  // Window
  {
    id: "minimize",
    label: "Minimize",
    category: "window",
    defaultKeys: [{ key: "m", ctrl: true }],
    enabled: true,
  },
  {
    id: "close_window",
    label: "Close Window",
    category: "window",
    defaultKeys: [{ key: "w", ctrl: true }],
    enabled: true,
  },
  {
    id: "quit_amus",
    label: "Quit AMUS",
    category: "window",
    defaultKeys: [{ key: "q", ctrl: true }],
    enabled: true,
  },
];

export interface GlobalShortcutAction {
  id: string;
  label: string;
  defaultBinding: ShortcutBinding | null;
  enabled: boolean;
}

export const GLOBAL_SHORTCUT_ACTIONS: GlobalShortcutAction[] = [
  {
    id: "global_play_pause",
    label: "Play / Pause",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_next_track",
    label: "Next Track",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_prev_track",
    label: "Previous Track",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_stop",
    label: "Stop",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_volume_up",
    label: "Volume Up",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_volume_down",
    label: "Volume Down",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_toggle_mute",
    label: "Toggle Mute",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_seek_forward",
    label: "Seek Forward",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_seek_backward",
    label: "Seek Backward",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_toggle_shuffle",
    label: "Toggle Shuffle",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_toggle_repeat",
    label: "Toggle Repeat",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_show_hide",
    label: "Show / Hide AMUS",
    defaultBinding: null,
    enabled: false,
  },
  {
    id: "global_show_miniplayer",
    label: "Show / Hide Mini Player",
    defaultBinding: null,
    enabled: false,
  },
];

let store: Awaited<ReturnType<typeof load>> | null = null;
let initialized = false;

export const customBindings = $state<Record<string, ShortcutBinding[]>>({});
export const disabledShortcuts = $state<Record<string, boolean>>({});
export const globalShortcutFlags = $state<Record<string, boolean>>({});
export const globalCustomBindings = $state<Record<string, ShortcutBinding>>({});
export const flags = $state({ ready: false });

export async function initShortcuts(): Promise<void> {
  if (initialized) return;
  initialized = true;
  store = await load("settings.json", { autoSave: true, defaults: {} });

  const saved = await store.get<Record<string, ShortcutBinding[]>>("shortcuts_custom");
  if (saved) Object.assign(customBindings, saved);

  const savedDisabled = await store.get<Record<string, boolean>>("shortcuts_disabled");
  if (savedDisabled) Object.assign(disabledShortcuts, savedDisabled);

  for (const g of GLOBAL_SHORTCUT_ACTIONS) {
    const val = await store.get<boolean>(`shortcuts_global_${g.id}`);
    globalShortcutFlags[g.id] = val !== undefined ? val : g.enabled;
  }

  const savedGlobalBindings = await store.get<Record<string, ShortcutBinding>>(
    "shortcuts_global_bindings",
  );
  if (savedGlobalBindings) Object.assign(globalCustomBindings, savedGlobalBindings);

  flags.ready = true;
  await updateTauriGlobalShortcuts();
}

function getStoreOrLoad(): Promise<Awaited<ReturnType<typeof load>>> {
  if (store) return Promise.resolve(store);
  return load("settings.json", { autoSave: true, defaults: {} });
}

export function getEffectiveBindings(action: ShortcutAction): ShortcutBinding[] {
  const custom = customBindings[action.id];
  if (custom === undefined) return action.defaultKeys;
  return custom;
}

export function isShortcutEnabled(actionId: string): boolean {
  return disabledShortcuts[actionId] !== true;
}

export function setDisabled(actionId: string, disabled: boolean): void {
  if (disabled) {
    disabledShortcuts[actionId] = true;
  } else {
    delete disabledShortcuts[actionId];
  }
  saveDisabled();
}

export async function setCustomBindings(
  actionId: string,
  bindings: ShortcutBinding[],
): Promise<void> {
  customBindings[actionId] = bindings;
  await saveCustom();
}

export async function addBinding(actionId: string, binding: ShortcutBinding): Promise<void> {
  const existing =
    customBindings[actionId] ?? ALL_SHORTCUTS.find((a) => a.id === actionId)?.defaultKeys ?? [];
  customBindings[actionId] = [...existing, binding];
  await saveCustom();
}

export async function removeSingleBinding(
  actionId: string,
  binding: ShortcutBinding,
): Promise<void> {
  const existing =
    customBindings[actionId] ?? ALL_SHORTCUTS.find((a) => a.id === actionId)?.defaultKeys ?? [];
  const updated = existing.filter((b) => !bindingsEqual(b, binding));
  customBindings[actionId] = updated;
  await saveCustom();
}

export async function resetCustomBindings(actionId: string): Promise<void> {
  delete customBindings[actionId];
  await saveCustom();
}

export async function resetAllBindings(): Promise<void> {
  for (const key of Object.keys(customBindings)) {
    delete customBindings[key];
  }
  for (const key of Object.keys(disabledShortcuts)) {
    delete disabledShortcuts[key];
  }
  for (const key of Object.keys(globalCustomBindings)) {
    delete globalCustomBindings[key];
  }
  for (const g of GLOBAL_SHORTCUT_ACTIONS) {
    globalShortcutFlags[g.id] = g.enabled;
  }
  await saveCustom();
  await saveDisabled();
  const s = await getStoreOrLoad();
  if (!store) store = s;
  await s.set("shortcuts_global_bindings", {});
  for (const g of GLOBAL_SHORTCUT_ACTIONS) {
    await s.set(`shortcuts_global_${g.id}`, g.enabled);
  }
  await s.save();
  await updateTauriGlobalShortcuts();
}

async function saveCustom(): Promise<void> {
  const s = await getStoreOrLoad();
  if (!store) store = s;
  await s.set("shortcuts_custom", { ...customBindings });
  await s.save();
}

async function saveDisabled(): Promise<void> {
  const s = await getStoreOrLoad();
  if (!store) store = s;
  await s.set("shortcuts_disabled", { ...disabledShortcuts });
  await s.save();
}

export function getEffectiveGlobalBinding(g: GlobalShortcutAction): ShortcutBinding | null {
  const custom = globalCustomBindings[g.id];
  return custom ?? g.defaultBinding;
}

export async function setGlobalShortcutEnabled(id: string, enabled: boolean): Promise<void> {
  globalShortcutFlags[id] = enabled;
  const s = await getStoreOrLoad();
  if (!store) store = s;
  await s.set(`shortcuts_global_${id}`, enabled);
  await s.save();
  await updateTauriGlobalShortcuts();
}

export async function setGlobalCustomBinding(id: string, binding: ShortcutBinding): Promise<void> {
  globalCustomBindings[id] = binding;
  globalShortcutFlags[id] = true;
  const s = await getStoreOrLoad();
  if (!store) store = s;
  await s.set("shortcuts_global_bindings", { ...globalCustomBindings });
  await s.set(`shortcuts_global_${id}`, true);
  await s.save();
  await updateTauriGlobalShortcuts();
}

export async function resetGlobalCustomBinding(id: string): Promise<void> {
  delete globalCustomBindings[id];
  const g = GLOBAL_SHORTCUT_ACTIONS.find((a) => a.id === id);
  const stillEnabled = g?.defaultBinding != null;
  globalShortcutFlags[id] = stillEnabled;
  const s = await getStoreOrLoad();
  if (!store) store = s;
  await s.set("shortcuts_global_bindings", { ...globalCustomBindings });
  await s.set(`shortcuts_global_${id}`, stillEnabled);
  await s.save();
  await updateTauriGlobalShortcuts();
}

export function matchBinding(e: KeyboardEvent, binding: ShortcutBinding): boolean {
  const key = e.key === " " ? "Space" : e.key;
  const keyMatch = key.toLowerCase() === binding.key.toLowerCase() || key === binding.key;
  if (!keyMatch) return false;

  const isMac = typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");

  const targetCtrl = !!binding.ctrl;
  const targetMeta = !!binding.meta;
  const targetShift = !!binding.shift;
  const targetAlt = !!binding.alt;

  const pressedCtrl = e.ctrlKey;
  const pressedMeta = e.metaKey;
  const pressedShift = e.shiftKey;
  const pressedAlt = e.altKey;

  if (isMac) {
    const hasPrimaryModifier = targetCtrl || targetMeta;
    const pressedPrimaryModifier = pressedCtrl || pressedMeta;
    if (hasPrimaryModifier !== pressedPrimaryModifier) return false;
  } else {
    if (targetCtrl !== pressedCtrl) return false;
    if (targetMeta !== pressedMeta) return false;
  }

  if (targetShift !== pressedShift) return false;
  if (targetAlt !== pressedAlt) return false;

  return true;
}

export function findAction(e: KeyboardEvent): ShortcutAction | null {
  if (isInputFocused()) return null;
  for (const action of ALL_SHORTCUTS) {
    if (!isShortcutEnabled(action.id)) continue;
    const bindings = getEffectiveBindings(action);
    for (const binding of bindings) {
      if (matchBinding(e, binding)) return action;
    }
  }
  return null;
}

export function isInputFocused(): boolean {
  const el = document.activeElement;
  if (!el) return false;
  const tag = el.tagName.toLowerCase();
  if (tag === "input" || tag === "textarea" || tag === "select") return true;
  if ((el as HTMLElement).isContentEditable) return true;
  if (el.getAttribute("role") === "textbox") return true;
  return false;
}

export function formatBinding(binding: ShortcutBinding): string[];
export function formatBinding(binding: ShortcutBinding, mode: "tauri"): string;
export function formatBinding(
  binding: ShortcutBinding,
  mode: "display" | "tauri" = "display",
): string[] | string {
  const parts: string[] = [];

  if (mode === "tauri") {
    if (binding.ctrl) parts.push("Control");
    if (binding.shift) parts.push("Shift");
    if (binding.alt) parts.push("Alt");
    if (binding.meta) parts.push("Command");
  } else {
    const isMac = typeof navigator !== "undefined" && navigator.userAgent.includes("Mac");
    if (isMac) {
      if (binding.ctrl || binding.meta) parts.push("Cmd");
      if (binding.shift) parts.push("Shift");
      if (binding.alt) parts.push("Option");
    } else {
      if (binding.ctrl) parts.push("Ctrl");
      if (binding.shift) parts.push("Shift");
      if (binding.alt) parts.push("Alt");
      if (binding.meta) parts.push("Win");
    }
  }

  if (mode === "tauri") {
    let key = binding.key;
    if (key === " ") key = "Space";
    else if (key.length === 1) key = key.toUpperCase();
    parts.push(key);
    return parts.join("+");
  }

  const display = KEY_DISPLAY[binding.key] ?? binding.key.toUpperCase();
  parts.push(display);
  return parts;
}

export function formatShortcut(action: ShortcutAction): string {
  return getEffectiveBindings(action)
    .map((b) => formatBinding(b).join(" + "))
    .join(", ");
}

export function bindingFromEvent(e: KeyboardEvent): ShortcutBinding | null {
  const key = e.key === " " ? "Space" : e.key;
  if (["Control", "Shift", "Alt", "Meta", "Process"].includes(key)) return null;
  return {
    key,
    ctrl: e.ctrlKey,
    shift: e.shiftKey,
    alt: e.altKey,
    meta: e.metaKey,
  };
}

export function findConflicts(actionId: string, newBindings: ShortcutBinding[]): string[] {
  const conflicts: string[] = [];
  for (const action of ALL_SHORTCUTS) {
    if (action.id === actionId) continue;
    if (!isShortcutEnabled(action.id)) continue;
    const existing = getEffectiveBindings(action);
    for (const eb of existing) {
      for (const nb of newBindings) {
        if (bindingsEqual(eb, nb)) {
          conflicts.push(action.label);
          break;
        }
      }
    }
  }
  for (const g of GLOBAL_SHORTCUT_ACTIONS) {
    if (g.id === actionId) continue;
    if (!globalShortcutFlags[g.id]) continue;
    const existing = getEffectiveGlobalBinding(g);
    if (!existing) continue;
    for (const nb of newBindings) {
      if (bindingsEqual(existing, nb)) {
        conflicts.push(g.label);
        break;
      }
    }
  }
  return conflicts;
}

export async function removeBindingFromAllOthers(
  actionId: string,
  binding: ShortcutBinding,
): Promise<void> {
  for (const action of ALL_SHORTCUTS) {
    if (action.id === actionId) continue;
    const existing = getEffectiveBindings(action);
    if (existing.some((b) => bindingsEqual(b, binding))) {
      const updated = existing.filter((b) => !bindingsEqual(b, binding));
      customBindings[action.id] = updated;
    }
  }

  let globalStoreChanged = false;
  for (const g of GLOBAL_SHORTCUT_ACTIONS) {
    if (g.id === actionId) continue;
    const existing = getEffectiveGlobalBinding(g);
    if (existing && bindingsEqual(existing, binding)) {
      if (globalCustomBindings[g.id]) {
        delete globalCustomBindings[g.id];
        globalStoreChanged = true;
      }
      globalShortcutFlags[g.id] = false;
      const s = await getStoreOrLoad();
      await s.set(`shortcuts_global_${g.id}`, false);
      globalStoreChanged = true;
    }
  }

  await saveCustom();
  if (globalStoreChanged) {
    const s = await getStoreOrLoad();
    if (!store) store = s;
    await s.set("shortcuts_global_bindings", { ...globalCustomBindings });
    await s.save();
  }
}

export function bindingsEqual(a: ShortcutBinding, b: ShortcutBinding): boolean {
  return (
    a.key.toLowerCase() === b.key.toLowerCase() &&
    !!a.ctrl === !!b.ctrl &&
    !!a.shift === !!b.shift &&
    !!a.alt === !!b.alt &&
    !!a.meta === !!b.meta
  );
}

let registeredTauriShortcuts: string[] = [];

export async function updateTauriGlobalShortcuts(): Promise<void> {
  try {
    const { unregister, register } = await import("@tauri-apps/plugin-global-shortcut");

    for (const shortcutStr of registeredTauriShortcuts) {
      try {
        await unregister(shortcutStr);
      } catch (e) {
        console.warn("Failed to unregister global shortcut:", shortcutStr, e);
      }
    }
    registeredTauriShortcuts = [];

    for (const g of GLOBAL_SHORTCUT_ACTIONS) {
      if (!globalShortcutFlags[g.id]) continue;
      const binding = getEffectiveGlobalBinding(g);
      if (!binding) continue;

      const shortcutStr = formatBinding(binding, "tauri");
      try {
        await register(shortcutStr, async () => {
          const fn = getHandler(g.id);
          if (fn) {
            await fn();
          }
        });
        registeredTauriShortcuts.push(shortcutStr);
      } catch (e) {
        console.error("Failed to register global shortcut:", shortcutStr, e);
      }
    }
  } catch (err) {
    console.error("Global shortcut plugin error:", err);
  }
}

export const handlerMap = new Map<string, () => void | Promise<void>>();

export function getHandler(actionId: string): (() => void | Promise<void>) | undefined {
  return handlerMap.get(actionId);
}
