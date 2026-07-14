import { vi } from "vitest";

if (typeof globalThis.requestAnimationFrame === "undefined") {
  globalThis.requestAnimationFrame = ((cb: FrameRequestCallback) => {
    return setTimeout(cb, 16) as unknown as number;
  }) as typeof globalThis.requestAnimationFrame;
  globalThis.cancelAnimationFrame = (id: number) => clearTimeout(id);
}

if (typeof globalThis.window === "undefined") {
  globalThis.window = {
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
  } as any;
}

if (typeof globalThis.navigator === "undefined") {
  globalThis.navigator = { userAgent: "node" } as any;
}

if (typeof globalThis.document === "undefined") {
  globalThis.document = {
    activeElement: null,
  } as any;
}

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
  convertFileSrc: vi.fn(() => "asset://localhost/asset.png"),
}));

const eventListeners = new Map<string, (payload: any) => void>();
export function emitEvent(event: string, payload: unknown) {
  const handler = eventListeners.get(event);
  if (handler) handler({ payload, event });
}

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((event: string, handler: (payload: any) => void) => {
    eventListeners.set(event, handler);
    return Promise.resolve(() => {
      eventListeners.delete(event);
    });
  }),
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  load: vi.fn(() =>
    Promise.resolve({
      get: vi.fn(() => Promise.resolve(undefined)),
      set: vi.fn(() => Promise.resolve()),
      save: vi.fn(() => Promise.resolve()),
    }),
  ),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-global-shortcut", () => ({
  register: vi.fn(),
  unregister: vi.fn(),
  isRegistered: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
  onUpdaterEvent: vi.fn(),
}));

vi.mock("@tauri-apps/api/path", () => ({
  appDataDir: vi.fn(() => Promise.resolve("/mock/appdata")),
}));
