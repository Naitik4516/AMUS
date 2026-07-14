import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import * as shortcuts from "./shortcuts.svelte";
import type { ShortcutBinding, ShortcutAction } from "./shortcuts.svelte";
import { load as storeLoad } from "@tauri-apps/plugin-store";

function binding(k: string, mods?: Partial<ShortcutBinding>): ShortcutBinding {
  return { key: k, ...mods };
}

function event(key: string, mods?: Record<string, boolean>): Record<string, unknown> {
  return { key, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, ...mods };
}

function resetState() {
  shortcuts.flags.ready = false;
  for (const key of Object.keys(shortcuts.customBindings))
    delete (shortcuts.customBindings as any)[key];
  for (const key of Object.keys(shortcuts.disabledShortcuts))
    delete (shortcuts.disabledShortcuts as any)[key];
  for (const key of Object.keys(shortcuts.globalShortcutFlags))
    delete (shortcuts.globalShortcutFlags as any)[key];
  for (const key of Object.keys(shortcuts.globalCustomBindings))
    delete (shortcuts.globalCustomBindings as any)[key];
  shortcuts.handlerMap.clear();
}

beforeEach(() => {
  resetState();
  vi.clearAllMocks();
  document.activeElement = null;
});

// ── Pure helpers ──────────────────────────────────────────────

describe("bindingsEqual", () => {
  it("returns true for identical bindings", () => {
    expect(shortcuts.bindingsEqual(binding("k"), binding("k"))).toBe(true);
  });

  it("is case-insensitive for key", () => {
    expect(shortcuts.bindingsEqual(binding("K"), binding("k"))).toBe(true);
  });

  it("returns false when modifier differs", () => {
    expect(shortcuts.bindingsEqual(binding("k", { ctrl: true }), binding("k"))).toBe(false);
  });

  it("coalesces undefined modifiers to false", () => {
    const a: ShortcutBinding = { key: "k", ctrl: true };
    const b: ShortcutBinding = { key: "k", ctrl: true, shift: undefined };
    expect(shortcuts.bindingsEqual(a, b)).toBe(true);
  });
});

describe("matchBinding", () => {
  it("returns true for exact key match", () => {
    expect(shortcuts.matchBinding(event("k"), binding("k"))).toBe(true);
  });

  it("returns true when ctrl is pressed and required", () => {
    expect(
      shortcuts.matchBinding(event("k", { ctrlKey: true }), binding("k", { ctrl: true })),
    ).toBe(true);
  });

  it("returns false when ctrl is required but not pressed", () => {
    expect(shortcuts.matchBinding(event("k"), binding("k", { ctrl: true }))).toBe(false);
  });

  it("returns true when shift is pressed and required", () => {
    expect(
      shortcuts.matchBinding(event("k", { shiftKey: true }), binding("k", { shift: true })),
    ).toBe(true);
  });

  it("returns false on key mismatch", () => {
    expect(shortcuts.matchBinding(event("a"), binding("b"))).toBe(false);
  });

  it("normalises Space key", () => {
    expect(shortcuts.matchBinding(event(" "), binding("Space"))).toBe(true);
  });

  it("is case-insensitive for key", () => {
    expect(shortcuts.matchBinding(event("K"), binding("k"))).toBe(true);
  });

  it("on non-Mac: meta must match exactly", () => {
    expect(shortcuts.matchBinding(event("k", { metaKey: true }), binding("k"))).toBe(false);
    expect(shortcuts.matchBinding(event("k"), binding("k", { meta: true }))).toBe(false);
    expect(
      shortcuts.matchBinding(event("k", { metaKey: true }), binding("k", { meta: true })),
    ).toBe(true);
  });
});

describe("isInputFocused", () => {
  it("returns false when no element is focused", () => {
    document.activeElement = null;
    expect(shortcuts.isInputFocused()).toBe(false);
  });

  it("returns true when input is focused", () => {
    document.activeElement = { tagName: "INPUT" } as any;
    expect(shortcuts.isInputFocused()).toBe(true);
  });

  it("returns true when textarea is focused", () => {
    document.activeElement = { tagName: "TEXTAREA" } as any;
    expect(shortcuts.isInputFocused()).toBe(true);
  });

  it("returns true for contentEditable elements", () => {
    document.activeElement = { tagName: "DIV", isContentEditable: true } as any;
    expect(shortcuts.isInputFocused()).toBe(true);
  });

  it("returns true for role=textbox elements", () => {
    document.activeElement = { tagName: "DIV", getAttribute: () => "textbox" } as any;
    expect(shortcuts.isInputFocused()).toBe(true);
  });
});

describe("bindingFromEvent", () => {
  it("extracts key and modifiers from event", () => {
    const e = event("ArrowRight", { ctrlKey: true, shiftKey: true });
    const b = shortcuts.bindingFromEvent(e)!;
    expect(b.key).toBe("ArrowRight");
    expect(b.ctrl).toBe(true);
    expect(b.shift).toBe(true);
  });

  it("returns null for modifier-only keys", () => {
    expect(shortcuts.bindingFromEvent(event("Control"))).toBeNull();
    expect(shortcuts.bindingFromEvent(event("Shift"))).toBeNull();
    expect(shortcuts.bindingFromEvent(event("Meta"))).toBeNull();
    expect(shortcuts.bindingFromEvent(event("Alt"))).toBeNull();
  });

  it("normalises space key", () => {
    const b = shortcuts.bindingFromEvent(event(" "))!;
    expect(b.key).toBe("Space");
  });
});

describe("formatBinding", () => {
  const origUA = navigator.userAgent;

  afterEach(() => {
    Object.defineProperty(navigator, "userAgent", { value: origUA, configurable: true });
  });

  function setMac(flag: boolean) {
    Object.defineProperty(navigator, "userAgent", {
      value: flag ? "Macintosh" : "Linux",
      configurable: true,
    });
  }

  it("display: Ctrl+K on non-Mac", () => {
    setMac(false);
    const parts = shortcuts.formatBinding(binding("k", { ctrl: true })) as string[];
    expect(parts.join(" + ")).toBe("Ctrl + K");
  });

  it("display: Cmd+K on Mac", () => {
    setMac(true);
    const parts = shortcuts.formatBinding(binding("k", { ctrl: true })) as string[];
    expect(parts.join(" + ")).toBe("Cmd + K");
  });

  it("display: Shift+Ctrl+K on non-Mac", () => {
    setMac(false);
    const parts = shortcuts.formatBinding(binding("k", { ctrl: true, shift: true })) as string[];
    expect(parts.join(" + ")).toBe("Ctrl + Shift + K");
  });

  it("display: Space uses KEY_DISPLAY", () => {
    setMac(false);
    const parts = shortcuts.formatBinding(binding("Space")) as string[];
    expect(parts.join(" + ")).toBe("Space");
  });

  it("tauri mode: returns single string", () => {
    const result = shortcuts.formatBinding(binding("k", { ctrl: true, shift: true }), "tauri");
    expect(result).toBe("Control+Shift+K");
  });

  it("tauri mode: Space mapped to Space", () => {
    const result = shortcuts.formatBinding(binding(" "), "tauri");
    expect(result).toBe("Space");
  });

  it("Mac display: Option modifier", () => {
    setMac(true);
    const parts = shortcuts.formatBinding(binding("a", { alt: true })) as string[];
    expect(parts.join(" + ")).toBe("Option + A");
  });

  it("non-Mac display: Alt modifier", () => {
    setMac(false);
    const parts = shortcuts.formatBinding(binding("a", { alt: true })) as string[];
    expect(parts.join(" + ")).toBe("Alt + A");
  });

  it("Win modifier on non-Mac, non-Windows (falls through)", () => {
    setMac(false);
    const parts = shortcuts.formatBinding(binding("r", { meta: true })) as string[];
    expect(parts.join(" + ")).toBe("Win + R");
  });
});

// ── Action lookup ─────────────────────────────────────────────

describe("findAction", () => {
  it("matches play_pause by space", () => {
    expect(shortcuts.findAction(event(" "))?.id).toBe("play_pause");
  });

  it("matches next_track by n", () => {
    expect(shortcuts.findAction(event("n"))?.id).toBe("next_track");
  });

  it("matches prev_track by b", () => {
    expect(shortcuts.findAction(event("b"))?.id).toBe("prev_track");
  });

  it("returns null when input is focused", () => {
    document.activeElement = { tagName: "INPUT" } as any;
    expect(shortcuts.findAction(event(" "))).toBeNull();
    document.activeElement = null;
  });

  it("skips disabled actions", () => {
    shortcuts.disabledShortcuts["play_pause"] = true;
    expect(shortcuts.findAction(event(" "))?.id).not.toBe("play_pause");
  });

  it("honours custom bindings", () => {
    shortcuts.customBindings["play_pause"] = [{ key: "p" }];
    // old Space bind should not match
    expect(shortcuts.findAction(event(" "))?.id).not.toBe("play_pause");
    // new P bind should match
    expect(shortcuts.findAction(event("p"))?.id).toBe("play_pause");
  });

  it("matches action with multiple default bindings", () => {
    // play_pause has default: [{key:"Space"}, {key:"k"}]
    expect(shortcuts.findAction(event("k"))?.id).toBe("play_pause");
  });
});

describe("getEffectiveBindings", () => {
  it("returns defaultKeys when no custom binding", () => {
    const action = shortcuts.ALL_SHORTCUTS.find((a) => a.id === "play_pause")!;
    const bindings = shortcuts.getEffectiveBindings(action);
    expect(bindings).toEqual(action.defaultKeys);
  });

  it("returns custom bindings when set", () => {
    const custom = [{ key: "z" }];
    shortcuts.customBindings["play_pause"] = custom;
    const action = shortcuts.ALL_SHORTCUTS.find((a) => a.id === "play_pause")!;
    expect(shortcuts.getEffectiveBindings(action)).toBe(custom);
  });
});

describe("isShortcutEnabled", () => {
  it("returns true by default", () => {
    expect(shortcuts.isShortcutEnabled("random")).toBe(true);
  });

  it("returns false when disabled", () => {
    shortcuts.disabledShortcuts["play_pause"] = true;
    expect(shortcuts.isShortcutEnabled("play_pause")).toBe(false);
  });
});

describe("formatShortcut", () => {
  it("formats shortcut for display", () => {
    const action = shortcuts.ALL_SHORTCUTS.find((a) => a.id === "play_pause")!;
    const result = shortcuts.formatShortcut(action);
    expect(result).toBe("Space, K");
  });

  it("reflects custom bindings", () => {
    shortcuts.customBindings["play_pause"] = [{ key: "p" }];
    const action = shortcuts.ALL_SHORTCUTS.find((a) => a.id === "play_pause")!;
    expect(shortcuts.formatShortcut(action)).toBe("P");
  });
});

// ── Conflict detection ────────────────────────────────────────

describe("findConflicts", () => {
  it("no conflicts for unique binding", () => {
    const conflicts = shortcuts.findConflicts("play_pause", [{ key: "ctrl+alt+x" }]);
    expect(conflicts).toEqual([]);
  });

  it("detects conflict with another local action", () => {
    // Space is bound to play_pause, check if adding Space to stop conflicts
    const conflicts = shortcuts.findConflicts("stop", [{ key: "Space" }]);
    expect(conflicts).toContain("Play / Pause");
  });

  it("skips self", () => {
    const conflicts = shortcuts.findConflicts("play_pause", [{ key: "Space" }]);
    expect(conflicts).not.toContain("Play / Pause");
  });

  it("skips disabled actions", () => {
    shortcuts.disabledShortcuts["play_pause"] = true;
    const conflicts = shortcuts.findConflicts("stop", [{ key: "Space" }]);
    expect(conflicts.filter((c) => c === "Play / Pause")).toEqual([]);
  });
});

// ── State mutations ───────────────────────────────────────────

describe("setDisabled", () => {
  it("adds to disabledShortcuts and persists", () => {
    shortcuts.setDisabled("next_track", true);
    expect(shortcuts.disabledShortcuts["next_track"]).toBe(true);

    shortcuts.setDisabled("next_track", false);
    expect(shortcuts.disabledShortcuts["next_track"]).toBeUndefined();
  });
});

describe("initShortcuts", () => {
  it("sets ready flag after loading", async () => {
    await shortcuts.initShortcuts();
    expect(shortcuts.flags.ready).toBe(true);
  });

  it("is idempotent", async () => {
    await shortcuts.initShortcuts();
    vi.mocked(storeLoad).mockClear();
    await shortcuts.initShortcuts();
    expect(storeLoad).not.toHaveBeenCalled();
  });
});

describe("getEffectiveGlobalBinding", () => {
  it("returns defaultBinding when no custom set", () => {
    const g = shortcuts.GLOBAL_SHORTCUT_ACTIONS.find((a) => a.id === "global_play_pause")!;
    expect(shortcuts.getEffectiveGlobalBinding(g)).toBeNull();
  });

  it("returns custom when set", () => {
    const custom = { key: "MediaPlayPause" };
    shortcuts.globalCustomBindings["global_play_pause"] = custom;
    const g = shortcuts.GLOBAL_SHORTCUT_ACTIONS.find((a) => a.id === "global_play_pause")!;
    expect(shortcuts.getEffectiveGlobalBinding(g)).toBe(custom);
  });
});
