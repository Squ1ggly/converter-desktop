<script lang="ts">
  import { onMount } from "svelte";
  import { ModeWatcher } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import type { Event as TauriEvent } from "@tauri-apps/api/event";
  import { getCurrentWebview, type DragDropEvent } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { open, type OpenDialogOptions } from "@tauri-apps/plugin-dialog";
  import { Toaster } from "$lib/components/ui/sonner/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import BusyState from "$lib/components/BusyState.svelte";
  import DropOverlay from "$lib/components/DropOverlay.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import FileList from "$lib/components/FileList.svelte";
  import HelpView from "$lib/components/HelpView.svelte";
  import QuitDialog from "$lib/components/QuitDialog.svelte";
  import SettingsPanel from "$lib/components/SettingsPanel.svelte";
  import SettingsView from "$lib/components/SettingsView.svelte";
  import Toolbar from "$lib/components/Toolbar.svelte";
  import {
    getCatalog,
    openOutput,
    revealOutput,
    type ConversionCatalog,
    type FormatOption,
  } from "$lib/backend";
  import { ConversionRun } from "$lib/conversion-run.svelte";
  import { errorMessageOf, showError } from "$lib/errors";
  import { FileAdder, type AddOutcome } from "$lib/file-adder.svelte";
  import { pluralise, revealInFolderLabel } from "$lib/formatting";
  import { PreferenceStore } from "$lib/preference-store.svelte";
  import { FileQueue } from "$lib/queue.svelte";
  import { UNKNOWN_PLATFORM, shortcutFor, type Shortcut } from "$lib/shortcuts";
  import type { AppView } from "$lib/views";

  const MAX_NAMES_IN_TOAST = 3;
  const WAIT_FOR_CONVERSION_MESSAGE = "Wait for the current conversion to finish.";

  const queue = new FileQueue();
  const preferenceStore = new PreferenceStore();
  const conversionRun = new ConversionRun(queue);
  const fileAdder = new FileAdder(queue);

  let catalog = $state<ConversionCatalog | null>(null);
  let startupError = $state<string | null>(null);
  let hasLoadedPreferences = $state(false);
  let isDraggingFiles = $state(false);
  let view = $state<AppView>("converter");
  let isQuitDialogOpen = $state(false);

  const platform = $derived(catalog?.platform ?? UNKNOWN_PLATFORM);
  const revealLabel = $derived(revealInFolderLabel(platform));
  const imageTargetLabel = $derived(labelFor(catalog?.imageOutputs, preferenceStore.values.imageFormat));
  const videoTargetLabel = $derived(labelFor(catalog?.videoOutputs, preferenceStore.values.videoFormat));

  function labelFor(options: FormatOption[] | undefined, formatId: string): string {
    const label = options?.find((option) => option.id === formatId)?.label ?? formatId.toUpperCase();
    return label;
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

  // A damaged settings file shouldn't stop the app; the defaults are used instead.
  async function loadPreferences(): Promise<void> {
    try {
      await preferenceStore.load();
    } catch {
      toast.error("Couldn't load your settings", { description: "Using the defaults for now." });
    }
  }

  async function initialise(): Promise<void> {
    try {
      const [loadedCatalog] = await Promise.all([getCatalog(), loadPreferences()]);
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
      toast.info(WAIT_FOR_CONVERSION_MESSAGE);
      return;
    }

    // Shows the list straight away, so a slow folder scan shows its progress.
    view = "converter";
    try {
      const outcome = await fileAdder.add(paths);
      reportAddOutcome(outcome);
    } catch (error) {
      showError("Couldn't add files", error);
    }
  }

  function reportAddOutcome(outcome: AddOutcome): void {
    if (outcome.kind === "busy") {
      toast.info(WAIT_FOR_CONVERSION_MESSAGE);
      return;
    }
    if (outcome.kind !== "added" || outcome.unsupportedNames.length === 0) {
      return;
    }

    const skippedNames = outcome.unsupportedNames;
    toast.warning(`Skipped ${pluralise(skippedNames.length, "unsupported file")}`, {
      description: describeSkippedFiles(skippedNames),
    });
  }

  async function chooseFiles(): Promise<void> {
    if (!catalog) {
      return;
    }

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
    await addPaths(chosenPaths);
  }

  async function chooseFolder(): Promise<void> {
    const chosenPaths = await pickPaths("Couldn't open the folder picker", {
      directory: true,
      multiple: true,
      title: "Choose folders",
    });
    await addPaths(chosenPaths);
  }

  // Returns no paths if the picker is cancelled or can't open.
  async function pickPaths(errorTitle: string, options: OpenDialogOptions & { multiple: true }): Promise<string[]> {
    try {
      const chosenPaths = await open(options);
      return chosenPaths ?? [];
    } catch (error) {
      showError(errorTitle, error);
      return [];
    }
  }

  async function cancelConversion(): Promise<void> {
    try {
      await conversionRun.cancel();
    } catch (error) {
      showError("Couldn't cancel the conversion", error);
    }
  }

  async function startConversion(): Promise<void> {
    if (fileAdder.isAdding) {
      return;
    }
    await conversionRun.start($state.snapshot(preferenceStore.values));
  }

  async function openConvertedFile(path: string): Promise<void> {
    try {
      await openOutput(path);
    } catch (error) {
      showError("Couldn't open the file", error);
    }
  }

  async function revealConvertedFile(path: string): Promise<void> {
    try {
      await revealOutput(path);
    } catch (error) {
      showError("Couldn't show the file", error);
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

  function toggleView(target: AppView): void {
    view = view === target ? "converter" : target;
  }

  // Number fields save on change, which a shortcut's preventDefault skips.
  // Blurring fires that change first, so the typed value is used.
  function commitFocusedField(): void {
    const focusedElement = document.activeElement;
    if (focusedElement instanceof HTMLInputElement) {
      focusedElement.blur();
    }
  }

  function convertFromAnyView(): void {
    commitFocusedField();
    view = "converter";
    void startConversion();
  }

  const SHORTCUT_ACTIONS: Record<Shortcut, () => void> = {
    open: () => void chooseFiles(),
    convert: convertFromAnyView,
    settings: () => toggleView("settings"),
    help: () => toggleView("help"),
  };

  function handleKeydown(event: KeyboardEvent): void {
    const shortcut = shortcutFor(event, platform);
    if (!shortcut) {
      return;
    }
    event.preventDefault();
    SHORTCUT_ACTIONS[shortcut]();
  }

  // Closing mid-conversion asks first, so nothing is left half-written.
  function guardWindowClose(event: { preventDefault: () => void }): void {
    if (!queue.isConverting) {
      return;
    }
    event.preventDefault();
    isQuitDialogOpen = true;
  }

  async function quitAfterCancelling(): Promise<void> {
    await conversionRun.cancel();
    await conversionRun.waitUntilFinished();
    await getCurrentWindow().destroy();
  }

  onMount(() => {
    void initialise();
    const stopDragDropPromise = getCurrentWebview().onDragDropEvent(handleDragDrop);
    const stopCloseGuardPromise = getCurrentWindow().onCloseRequested(guardWindowClose);

    return () => {
      void stopDragDropPromise.then((stopListening) => stopListening());
      void stopCloseGuardPromise.then((stopListening) => stopListening());
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
      {view}
      isMac={platform === "macos"}
      hasFiles={!queue.isEmpty}
      isConverting={queue.isConverting}
      onAddFiles={chooseFiles}
      onClear={() => queue.clear()}
      onToggleView={toggleView}
      onBack={() => (view = "converter")}
    />

    <main class="min-h-0 flex-1">
      {#if startupError}
        <p class="text-destructive p-8 text-sm select-text" role="alert">Couldn't start: {startupError}</p>
      {:else if view === "help"}
        <HelpView />
      {:else if !catalog}
        <BusyState label="Loading" />
      {:else if catalog && view === "settings"}
        <SettingsView {catalog} bind:preferences={preferenceStore.values} isConverting={queue.isConverting} />
      {:else if catalog && queue.isEmpty && fileAdder.isAdding}
        <BusyState label="Adding files" />
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
            isAdding={fileAdder.isAdding}
            {imageTargetLabel}
            {videoTargetLabel}
            {revealLabel}
            onRemove={(itemId) => queue.remove(itemId)}
            onOpen={openConvertedFile}
            onReveal={revealConvertedFile}
          />
          <SettingsPanel
            {catalog}
            bind:preferences={preferenceStore.values}
            hasImages={queue.images.length > 0}
            hasVideos={queue.videos.length > 0}
            fileCount={queue.items.length}
            finishedCount={queue.summary.finishedCount}
            isConverting={queue.isConverting}
            isAdding={fileAdder.isAdding}
            isCancelling={conversionRun.isCancelling}
            isRerun={queue.hasFinishedRun}
            {platform}
            onConvert={startConversion}
            onCancel={cancelConversion}
          />
        </div>
      {/if}
    </main>
  </div>
</Tooltip.Provider>

{#if isDraggingFiles}
  <DropOverlay />
{/if}

<QuitDialog bind:open={isQuitDialogOpen} onQuit={quitAfterCancelling} />
