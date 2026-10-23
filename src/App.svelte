<script lang="ts">
  import { onMount } from "svelte";
  import type { Event as TauriEvent } from "@tauri-apps/api/event";
  import { getCurrentWebview, type DragDropEvent } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import {
    cancelConversion,
    convertFiles,
    getCatalog,
    inspectPaths,
    openOutput,
    revealOutput,
    type ConversionCatalog,
    type FormatOption,
  } from "./lib/backend";
  import { isPrimaryModifierPressed, modifierKeyLabel, pluralise, revealInFolderLabel } from "./lib/formatting";
  import { PreferenceStore } from "./lib/preferences.svelte";
  import { FileQueue } from "./lib/queue.svelte";
  import { buildConversionRequests } from "./lib/requests";
  import DropOverlay from "./components/DropOverlay.svelte";
  import EmptyState from "./components/EmptyState.svelte";
  import FileList from "./components/FileList.svelte";
  import SettingsPanel from "./components/SettingsPanel.svelte";
  import Toast from "./components/Toast.svelte";
  import Toolbar from "./components/Toolbar.svelte";

  const TOAST_DURATION_MS = 6000;
  const MAX_NAMES_IN_TOAST = 3;

  const queue = new FileQueue();
  const preferenceStore = new PreferenceStore();

  let catalog = $state<ConversionCatalog | null>(null);
  let startupError = $state<string | null>(null);
  let hasLoadedPreferences = $state(false);
  let isDraggingFiles = $state(false);
  let toastMessage = $state<string | null>(null);
  let activeBatchIds = $state<string[]>([]);
  let isCancelling = $state(false);

  const platform = $derived(catalog?.platform ?? "");
  const modifierLabel = $derived(modifierKeyLabel(platform));
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

  // A saved choice may not exist on this machine (HEIC output is macOS-only).
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

  function showToast(message: string): void {
    toastMessage = message;
  }

  function describeSkippedFiles(skippedNames: string[]): string {
    const listedNames = skippedNames.slice(0, MAX_NAMES_IN_TOAST).join(", ");
    const remainingCount = skippedNames.length - MAX_NAMES_IN_TOAST;
    const remainder = remainingCount > 0 ? ` and ${remainingCount} more` : "";

    const description = `Skipped ${pluralise(skippedNames.length, "file")} that aren't images or videos: ${listedNames}${remainder}.`;
    return description;
  }

  async function addPaths(paths: string[]): Promise<void> {
    if (paths.length === 0) {
      return;
    }
    if (queue.isConverting) {
      showToast("Wait for the current conversion to finish before adding more files.");
      return;
    }

    try {
      const inspectedFiles = await inspectPaths(paths);
      queue.startNextBatchIfFinished();
      const skippedNames = queue.add(inspectedFiles);

      if (skippedNames.length > 0) {
        showToast(describeSkippedFiles(skippedNames));
      }
    } catch (error) {
      showToast(`Couldn't add those files: ${errorMessageOf(error)}`);
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
    const chosenPaths = await open({ directory: true, multiple: true, title: "Choose folders to convert" });
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
      showToast(errorMessageOf(error));
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

  // Save preferences whenever they change, but never before the saved ones
  // have loaded (that would overwrite them with the defaults).
  $effect(() => {
    if (!hasLoadedPreferences) {
      return;
    }
    preferenceStore.persist();
  });

  $effect(() => {
    if (!toastMessage) {
      return;
    }

    const dismissTimer = setTimeout(() => (toastMessage = null), TOAST_DURATION_MS);
    return () => clearTimeout(dismissTimer);
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="app">
  <Toolbar
    isMac={platform === "macos"}
    hasFiles={!queue.isEmpty}
    isConverting={queue.isConverting}
    addShortcutLabel={modifierLabel}
    onAddFiles={chooseFiles}
    onClear={() => queue.clear()}
  />

  <main class="content">
    {#if startupError}
      <p class="startup-error selectable" role="alert">The app couldn't start: {startupError}</p>
    {:else if catalog && queue.isEmpty}
      <EmptyState
        imageExtensions={catalog.imageInputExtensions}
        videoExtensions={catalog.videoInputExtensions}
        addShortcutLabel={modifierLabel}
        onChooseFiles={chooseFiles}
        onChooseFolder={chooseFolder}
      />
    {:else if catalog}
      <div class="workspace">
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
          shortcutLabel={modifierLabel}
          onConvert={startConversion}
          onCancel={cancelActiveConversion}
        />
      </div>
    {/if}
  </main>

  {#if isDraggingFiles}
    <DropOverlay />
  {/if}

  {#if toastMessage}
    <Toast message={toastMessage} onDismiss={() => (toastMessage = null)} />
  {/if}
</div>

<style>
  .app {
    display: grid;
    grid-template-rows: var(--toolbar-height) 1fr;
    height: 100%;
  }

  .content {
    min-height: 0;
  }

  .workspace {
    display: grid;
    grid-template-columns: 1fr var(--settings-panel-width);
    height: 100%;
    min-height: 0;
  }

  .startup-error {
    padding: 32px;
    color: var(--color-danger);
  }
</style>
