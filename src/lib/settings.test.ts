import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";
import { load } from "@tauri-apps/plugin-store";

const settingsMod = await import("./settings.svelte");

beforeEach(() => {
  settingsMod.flags.ready = false;
  vi.clearAllMocks();
});

describe("initSettings", () => {
  it("loads store and sets defaults for all keys", async () => {
    await settingsMod.initSettings();
    expect(settingsMod.flags.ready).toBe(true);
    expect(settingsMod.settings.realtimeSync).toBe(true);
    expect(settingsMod.settings.syncOnStartup).toBe(true);
    expect(settingsMod.settings.autoCheckUpdates).toBe(true);
    expect(settingsMod.settings.autoplayEnabled).toBe(true);
    expect(settingsMod.settings.osMediaControls).toBe(true);
  });

  it("is idempotent — second call does not re-invoke load", async () => {
    await settingsMod.initSettings();
    vi.mocked(load).mockClear();
    await settingsMod.initSettings();
    expect(load).not.toHaveBeenCalled();
  });
});

describe("setSetting", () => {
  it("updates state immediately", async () => {
    await settingsMod.setSetting("realtimeSync", false);
    expect(settingsMod.settings.realtimeSync).toBe(false);

    await settingsMod.setSetting("autoCheckUpdates", false);
    expect(settingsMod.settings.autoCheckUpdates).toBe(false);
  });
});

describe("getSetting", () => {
  it("returns default when no stored value exists", async () => {
    const val = await settingsMod.getSetting("realtimeSync", true);
    expect(val).toBe(true);
  });
});
