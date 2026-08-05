import { check, type Update } from "@tauri-apps/plugin-updater";
import { getVersion } from "@tauri-apps/api/app";
import { relaunch } from "@tauri-apps/plugin-process";
import { message } from "@tauri-apps/plugin-dialog";
import { getUpdateInstallSupport } from "./commands.svelte";

class UpdateManager {
  updateAvailable = $state<Update | null>(null);
  currentVersion = $state("");
  checking = $state(false);
  downloading = $state(false);
  downloadProgress = $state(0);

  private installSupported: boolean | null = null;

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

  private async canInstallUpdate(): Promise<boolean> {
    if (this.installSupported === null) {
      try {
        this.installSupported = await getUpdateInstallSupport();
      } catch (error) {
        console.warn("Failed to determine update install support:", error);
        this.installSupported = true;
      }
    }
    return this.installSupported;
  }

  async downloadAndInstall(): Promise<void> {
    if (!this.updateAvailable || this.downloading) return;

    if (!(await this.canInstallUpdate())) {
      await message(
        "Auto-update isn't supported for package-manager installs of AMUS on Arch Linux.\n\n" +
          "Please update using your package manager instead:\n\n" +
          "    paru -S amus\n    yay -S amus",
        { title: "Update via package manager", kind: "info" },
      );
      return;
    }

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
    } catch (error) {
      const reason = error instanceof Error ? error.message : String(error);
      console.error("Update install failed:", error);
      throw new Error(`Failed to install update: ${reason}`);
    } finally {
      this.downloading = false;
    }
  }
}

export const updater = new UpdateManager();
