import { check, type Update } from "@tauri-apps/plugin-updater";
import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";

class UpdateManager {
  updateAvailable = $state<Update | null>(null);
  currentVersion = $state("");
  checking = $state(false);
  downloading = $state(false);
  downloadProgress = $state(0);

  async loadCurrentVersion() {
    if (!this.currentVersion) {
      this.currentVersion = await getVersion();
    }
  }

  async checkForUpdates(): Promise<boolean> {
    await this.loadCurrentVersion();
    this.checking = true;
    try {
      const update = await check();
      this.updateAvailable = update ?? null;
      return update !== null;
    } finally {
      this.checking = false;
    }
  }

  async downloadAndInstall(): Promise<void> {
    if (!this.updateAvailable) return;
    this.downloading = true;
    this.downloadProgress = 0;
    try {
      await this.updateAvailable.downloadAndInstall((event) => {
        switch (event.event) {
          case "Started":
            this.downloadProgress = 0;
            break;
          case "Progress":
            this.downloadProgress += event.data.chunkLength;
            break;
          case "Finished":
            this.downloadProgress = 100;
            break;
        }
      });
      await relaunch();
    } finally {
      this.downloading = false;
    }
  }
}

export const updater = new UpdateManager();
