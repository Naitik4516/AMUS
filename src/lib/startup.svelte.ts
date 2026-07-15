import { invoke } from "@tauri-apps/api/core";

class StartupStore {
  error = $state<string | null>(null);
  checked = $state(false);

  async check() {
    try {
      const err = await invoke<string | null>("get_startup_status");
      this.error = err;
    } catch (e) {
      this.error = `Failed to communicate with backend: ${e}`;
    } finally {
      this.checked = true;
    }
  }
}

export const startup = new StartupStore();
