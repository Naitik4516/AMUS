import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export async function importAudioLibrary() {
  const selected = await open({
    directory: true,
    multiple: true,
    title: "Select Audio Library Folder",
  });

  if (selected) {
    for (const path of selected) {
      await invoke("add_source", { path });
    }
  }
  await invoke("scan_library");
}

export async function getSourceDirs(): Promise<string[]> {
  return invoke("get_source_dirs");
}

export async function removeSource(path: string): Promise<void> {
  await invoke("remove_source", { path });
}

export async function scanLibrary(): Promise<void> {
  await invoke("scan_library");
}

export async function refreshWatcher(): Promise<void> {
  await invoke("refresh_watcher");
}
