// Converts real files in the built app, through the real Rust commands and
// the bundled FFmpeg, then checks the files it wrote.
import { readdir, rm } from "node:fs/promises";
import path from "node:path";
import { $, browser, expect } from "@wdio/globals";
import {
  createSampleMedia,
  decodesWithoutErrors,
  hasSignature,
  MP4_SIGNATURE,
  WEBP_SIGNATURE,
  type SampleMedia,
} from "./media-fixtures.ts";

const CONVERSION_TIMEOUT_MS = 60_000;

declare global {
  interface Window {
    __TAURI_INTERNALS__: {
      invoke(command: string, commandArguments: { event: string; payload: object }): Promise<void>;
    };
  }
}

// WebDriver can't use native file dialogs, so files are added as a drop is:
// by emitting the webview's drag-drop event. The app's own listener handles
// it and calls the real inspect_paths command.
async function dropFiles(paths: string[]): Promise<void> {
  await browser.execute(
    (droppedPaths) =>
      window.__TAURI_INTERNALS__.invoke("plugin:event|emit", {
        event: "tauri://drag-drop",
        payload: { paths: droppedPaths, position: { x: 0, y: 0 } },
      }),
    paths,
  );
}

async function clearFileList(): Promise<void> {
  const clearButton = $("button=Clear");
  if (await clearButton.isExisting()) {
    await clearButton.click();
  }
}

describe("Converter", () => {
  let sampleMedia: SampleMedia;

  before(async () => {
    sampleMedia = await createSampleMedia();
    await expect($("div*=Drop images or videos")).toBeDisplayed();
  });

  afterEach(async () => {
    await clearFileList();
  });

  after(async () => {
    await rm(sampleMedia.folder, { recursive: true, force: true });
  });

  it("converts an image and a video next to the originals", async () => {
    await dropFiles([sampleMedia.photoPath, sampleMedia.videoPath, sampleMedia.textPath]);
    await expect($("h2=2 files")).toBeDisplayed();
    await expect($("div*=Skipped 1 unsupported file")).toBeDisplayed();

    await $("button=WebP").click();
    await $("button*=Convert 2 files").click();
    await expect($("h2=2 files converted")).toBeDisplayed({ wait: CONVERSION_TIMEOUT_MS });

    const photoOutputPath = path.join(sampleMedia.folder, "photo.webp");
    const videoOutputPath = path.join(sampleMedia.folder, "clip.mp4");
    expect(await hasSignature(photoOutputPath, WEBP_SIGNATURE)).toBe(true);
    expect(await hasSignature(videoOutputPath, MP4_SIGNATURE)).toBe(true);
    expect(await decodesWithoutErrors(photoOutputPath)).toBe(true);
    expect(await decodesWithoutErrors(videoOutputPath)).toBe(true);
  });

  it("shows the error for a file FFmpeg can't read and leaves no output behind", async () => {
    await dropFiles([sampleMedia.brokenVideoPath]);

    await $("button*=Convert 1 file").click();
    await expect($("h2=0 files converted")).toBeDisplayed({ wait: CONVERSION_TIMEOUT_MS });

    await expect($("span*=Failed")).toBeDisplayed();
    await expect($('[role="alert"]')).toBeDisplayed();
    const folderContents = await readdir(sampleMedia.folder);
    const brokenOutputs = folderContents.filter((fileName) => fileName.startsWith("broken ("));
    expect(brokenOutputs).toEqual([]);
  });
});
