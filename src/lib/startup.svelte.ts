import { invoke } from "@tauri-apps/api/core";

class StartupStore {
  error = $state<string | null>(null);
  checked = $state(false);

  async check() {
    const maxAttempts = 10;
    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      try {
        const err = await invoke<string | null>("get_startup_status");
        this.error = err;
        break;
      } catch (e) {
        if (attempt === maxAttempts - 1) {
          this.error = `Failed to communicate with backend: ${e}`;
        } else {
          await new Promise((resolve) => setTimeout(resolve, 200 + attempt * 100));
        }
      }
    }
    this.checked = true;
  }
}

export const startup = new StartupStore();
