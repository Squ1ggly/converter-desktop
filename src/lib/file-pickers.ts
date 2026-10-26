// The native file and folder pickers. Both return no paths if cancelled or if
// the picker can't open, in which case the error is shown.
import { open, type OpenDialogOptions } from "@tauri-apps/plugin-dialog";
import type { ConversionCatalog } from "$lib/backend";
import { showError } from "$lib/errors";

async function pickPaths(errorTitle: string, options: OpenDialogOptions & { multiple: true }): Promise<string[]> {
  try {
    const chosenPaths = await open(options);
    return chosenPaths ?? [];
  } catch (error) {
    showError(errorTitle, error);
    return [];
  }
}

export async function pickFiles(catalog: ConversionCatalog): Promise<string[]> {
  const supportedExtensions = [...catalog.imageInputExtensions, ...catalog.videoInputExtensions];
  const chosenPaths = await pickPaths("Couldn't open the file picker", {
    multiple: true,
    title: "Choose images or videos",
    filters: [
      { name: "Images and videos", extensions: supportedExtensions },
      { name: "Images", extensions: catalog.imageInputExtensions },
      { name: "Videos", extensions: catalog.videoInputExtensions },
    ],
  });
  return chosenPaths;
}

export async function pickFolders(): Promise<string[]> {
  const chosenPaths = await pickPaths("Couldn't open the folder picker", {
    directory: true,
    multiple: true,
    title: "Choose folders",
  });
  return chosenPaths;
}
