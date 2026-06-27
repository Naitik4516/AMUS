import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

export async function selectAndUploadImage(
  type: "cover" | "artist" | "banner",
): Promise<string | null> {
  const selected = await open({
    multiple: false,
    title: "Select Image",
    filters: [
      {
        name: "Images",
        extensions: ["png", "jpg", "jpeg", "webp", "bmp"],
      },
    ],
  });

  if (!selected) return null;

  return invoke<string>("save_image", {
    sourcePath: selected,
    imageType: type,
  });
}
