// Opening a converted file, or showing it in the file manager.
import { openOutput, revealOutput } from "$lib/backend";
import { showError } from "$lib/errors";

export async function openConvertedFile(path: string): Promise<void> {
  try {
    await openOutput(path);
  } catch (error) {
    showError("Couldn't open the file", error);
  }
}

export async function revealConvertedFile(path: string): Promise<void> {
  try {
    await revealOutput(path);
  } catch (error) {
    showError("Couldn't show the file", error);
  }
}
