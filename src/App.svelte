<script lang="ts">
  import { onMount } from "svelte";
  import { ModeWatcher } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import type { Event as TauriEvent } from "@tauri-apps/api/event";
  import { getCurrentWebview, type DragDropEvent } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import DropOverlay from "$lib/components/DropOverlay.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import FileList from "$lib/components/FileList.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import {
    cancelConversion,
    convertFiles,
    getCatalog,
    inspectPaths,
    openOutput,
    revealOutput,
    type ConversionCatalog,
    type FormatOption,
  } from "$lib/backend";
  import { isPrimaryModifierPressed, pluralise, revealInFolderLabel } from "$lib/formatting";
  import { PreferenceStore } from "$lib/preferences.svelte";
  import { FileQueue } from "$lib/queue.svelte";
  import { buildConversionRequests } from "$lib/requests";

  const MAX_NAMES_IN_TOAST = 3;

  const queue = new FileQueue();
  const preferenceStore = new PreferenceStore();

  let catalog = $state<ConversionCatalog | null>(null);
  let startupError = $state<string | null>(null);
  let hasLoadedPreferences = $state(false);
  let isDraggingFiles = $state(false);
  let activeBatchIds = $state<string[]>([]);
  let isCancelling = $state(false);

  const platform = $derived(catalog?.platform ?? "");
  const revealLabel = $derived(revealInFolderLabel(platform));
  const imageTargetLabel = $derived(labelFor(catalog?.imageOutputs, preferenceStore.values.imageFormat));
  const videoTargetLabel = $derived(labelFor(catalog?.videoOutputs, preferenceStore.values.videoFormat));

  function labelFor(options: FormatOption[] | undefined, formatId: string): string {
    const label = options?.find((option) => option.id === formatId)?.label ?? formatId.toUpperCase();
    return label;
  }

  function errorMessageOf(error: unknown): string {
    const message = error instanceof Error ? error.message : String(error);
    return message;
  }

  // A saved format may not exist on this machine (HEIC output is macOS only).
  function fallBackToAvailableFormats(loadedCatalog: ConversionCatalog): void {
    const preferences = preferenceStore.values;
    const isImageFormatAvailable = loadedCatalog.imageOutputs.some((option) => option.id === preferences.imageFormat);
    const isVideoFormatAvailable = loadedCatalog.videoOutputs.some((option) => option.id === preferences.videoFormat);

    if (!isImageFormatAvailable && loadedCatalog.imageOutputs[0]) {
      preferences.imageFormat = loadedCatalog.imageOutputs[0].id;
    }
    if (!isVideoFormatAvailable && loadedCatalog.videoOutputs[0]) {
      preferences.videoFormat = loadedCatalog.videoOutputs[0].id;
    }
  }

  async function initialise(): Promise<void> {
    try {
      const [loadedCatalog] = await Promise.all([getCatalog(), preferenceStore.load()]);
      fallBackToAvailableFormats(loadedCatalog);
      catalog = loadedCatalog;
      hasLoadedPreferences = true;
    } catch (error) {
      startupError = errorMessageOf(error);
    }
  }

  function describeSkippedFiles(skippedNames: string[]): string {
    const listedNames = skippedNames.slice(0, MAX_NAMES_IN_TOAST).join(", ");
    const remainingCount = skippedNames.length - MAX_NAMES_IN_TOAST;
    const remainder = remainingCount > 0 ? ` and ${remainingCount} more` : "";

    const description = `${listedNames}${remainder}`;
    return description;
  }

  async function addPaths(paths: string[]): Promise<void> {
    if (paths.length === 0) {
      return;
    }
    if (queue.isConverting) {
      toast.info("Wait for the current conversion to finish.");
      return;
    }

    try {
      const inspectedFiles = await inspectPaths(paths);
      queue.startNextBatchIfFinished();
      const skippedNames = queue.add(inspectedFiles);

      if (skippedNames.length > 0) {
        toast.warning(`Skipped ${pluralise(skippedNames.length, "unsupported file")}`, {
          description: describeSkippedFiles(skippedNames),
        });
      }
    } catch (error) {
      toast.error("Couldn't add files", { description: errorMessageOf(error) });
    }
  }

  async function chooseFiles(): Promise<void> {
    if (!catalog) {
      return;
    }

    const supportedExtensions = [...catalog.imageInputExtensions, ...catalog.videoInputExtensions];
    const chosenPaths = await open({
      multiple: true,
      title: "Choose images or videos",
      filters: [
        { name: "Images and videos", extensions: supportedExtensions },
        { name: "Images", extensions: catalog.imageInputExtensions },
        { name: "Videos", extensions: catalog.videoInputExtensions },
      ],
    });
    await addPaths(chosenPaths ?? []);
  }

  async function chooseFolder(): Promise<void> {
    const chosenPaths = await open({ directory: true, multiple: true, title: "Choose folders" });
    await addPaths(chosenPaths ?? []);
  }

  async function startConversion(): Promise<void> {
    if (queue.isEmpty || queue.isConverting) {
      return;
    }

    queue.prepareForRun();
    const requests = buildConversionRequests(queue.items, $state.snapshot(preferenceStore.values));
    activeBatchIds = requests.map((request) => request.batchId);

    try {
      const batchRuns = requests.map((request) => convertFiles(request, (event) => queue.applyEvent(event)));
      await Promise.all(batchRuns);
    } catch (error) {
      queue.markUnfinishedAsFailed(errorMessageOf(error));
    } finally {
      activeBatchIds = [];
      isCancelling = false;
    }
  }

  async function cancelActiveConversion(): Promise<void> {
    isCancelling = true;
    await Promise.all(activeBatchIds.map((batchId) => cancelConversion(batchId)));
  }

  async function runFileAction(action: (path: string) => Promise<void>, path: string): Promise<void> {
    try {
      await action(path);
    } catch (error) {
      toast.error(errorMessageOf(error));
    }
  }

  function handleDragDrop(event: TauriEvent<DragDropEvent>): void {
    const dragEvent = event.payload;

    if (dragEvent.type === "enter") {
      isDraggingFiles = true;
      return;
    }
    if (dragEvent.type === "leave") {
      isDraggingFiles = false;
      return;
    }
    if (dragEvent.type === "drop") {
      isDraggingFiles = false;
      void addPaths(dragEvent.paths);
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (!isPrimaryModifierPressed(event, platform)) {
      return;
    }

    if (event.key.toLowerCase() === "o") {
      event.preventDefault();
      void chooseFiles();
      return;
    }
    if (event.key === "Enter") {
      event.preventDefault();
      void startConversion();
    }
  }

  onMount(() => {
    void initialise();
    const stopListeningPromise = getCurrentWebview().onDragDropEvent(handleDragDrop);

    return () => {
      void stopListeningPromise.then((stopListening) => stopListening());
    };
  });

  // Save on every change, but only after the saved values have loaded.
  $effect(() => {
    if (!hasLoadedPreferences) {
      return;
    }
    preferenceStore.persist();
  });
</script>

<svelte:window onkeydown={handleKeydown} />
<ModeWatcher />
<Toaster position="bottom-center" />

<Tooltip.Provider>
  <div class="flex h-full flex-col">
    <Toolbar
      isMac={platform === "macos"}
      hasFiles={!queue.isEmpty}
      isConverting={queue.isConverting}
      onAddFiles={chooseFiles}
      onClear={() => queue.clear()}
    />

    <main class="min-h-0 flex-1">
      {#if startupError}
        <p class="text-destructive p-8 text-sm select-text" role="alert">Couldn't start: {startupError}</p>
      {:else if catalog && queue.isEmpty}
        <EmptyState
          imageExtensions={catalog.imageInputExtensions}
          videoExtensions={catalog.videoInputExtensions}
          {platform}
          onChooseFiles={chooseFiles}
          onChooseFolder={chooseFolder}
        />
      {:else if catalog}
        <div class="grid h-full min-h-0 grid-cols-[1fr_22rem]">
          <FileList
            {queue}
            {imageTargetLabel}
            {videoTargetLabel}
            {revealLabel}
            onRemove={(itemId) => queue.remove(itemId)}
            onOpen={(path) => runFileAction(openOutput, path)}
            onReveal={(path) => runFileAction(revealOutput, path)}
          />
          <SettingsPanel
            {catalog}
            bind:preferences={preferenceStore.values}
            hasImages={queue.images.length > 0}
            hasVideos={queue.videos.length > 0}
            fileCount={queue.items.length}
            finishedCount={queue.summary.finishedCount}
            isConverting={queue.isConverting}
            {isCancelling}
            isRerun={queue.hasFinishedRun}
            {platform}
            onConvert={startConversion}
            onCancel={cancelActiveConversion}
          />
        </div>
      {/if}
    </main>
  </div>
</Tooltip.Provider>

{#if isDraggingFiles}
  <DropOverlay />
{/if}
