<script lang="ts">
  import { onMount } from "svelte";
  import { ModeWatcher } from "mode-watcher";
  import { toast } from "svelte-sonner";
  import type { Event as TauriEvent } from "@tauri-apps/api/event";
  import { getCurrentWebview, type DragDropEvent } from "@tauri-apps/api/webview";
  import { getCurrentWindow } from "@tauri-apps/api/window";
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
  import { getCatalog, type ConversionCatalog, type FormatOption } from "$lib/backend";
  import { ConversionRun } from "$lib/conversion-run.svelte";
  import { openConvertedFile, revealConvertedFile } from "$lib/converted-files";
  import { errorMessageOf, showError } from "$lib/errors";
  import { FileAdder, WAIT_FOR_CONVERSION_MESSAGE, reportAddOutcome } from "$lib/file-adder.svelte";
  import { pickFiles, pickFolders } from "$lib/file-pickers";
  import { revealInFolderLabel } from "$lib/formatting";
  import { PreferenceStore } from "$lib/preference-store.svelte";
  import { QuitGuard } from "$lib/quit-guard.svelte";
  import { FileQueue } from "$lib/queue.svelte";
  import { UNKNOWN_PLATFORM, shortcutFor, type Shortcut } from "$lib/shortcuts";
  import { HELP_REQUESTED_EVENT, type AppView } from "$lib/views";

  const queue = new FileQueue();
  const preferenceStore = new PreferenceStore();
  const conversionRun = new ConversionRun(queue);
  const fileAdder = new FileAdder(queue);
  const quitGuard = new QuitGuard(queue, conversionRun);

  let catalog = $state<ConversionCatalog | null>(null);
  let startupError = $state<string | null>(null);
  let hasLoadedPreferences = $state(false);
  let isDraggingFiles = $state(false);
  let view = $state<AppView>("converter");

  const platform = $derived(catalog?.platform ?? UNKNOWN_PLATFORM);
  const revealLabel = $derived(revealInFolderLabel(platform));
  const imageTargetLabel = $derived(labelFor(catalog?.imageOutputs, preferenceStore.values.imageFormat));
  const videoTargetLabel = $derived(labelFor(catalog?.videoOutputs, preferenceStore.values.videoFormat));

  function labelFor(options: FormatOption[] | undefined, formatId: string): string {
    const label = options?.find((option) => option.id === formatId)?.label ?? formatId.toUpperCase();
    return label;
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
      preferenceStore.useAvailableFormats(loadedCatalog);
      catalog = loadedCatalog;
      hasLoadedPreferences = true;
    } catch (error) {
      startupError = errorMessageOf(error);
    }
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

  async function chooseFiles(): Promise<void> {
    if (!catalog) {
      return;
    }

    const chosenPaths = await pickFiles(catalog);
    await addPaths(chosenPaths);
  }

  async function chooseFolder(): Promise<void> {
    const chosenPaths = await pickFolders();
    await addPaths(chosenPaths);
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
    // A held key repeats; each shortcut acts once. The quit dialog has its own keys.
    if (event.repeat || quitGuard.isDialogOpen) {
      return;
    }

    const shortcut = shortcutFor(event, platform);
    if (!shortcut) {
      return;
    }
    event.preventDefault();
    SHORTCUT_ACTIONS[shortcut]();
  }

  onMount(() => {
    void initialise();
    const stopDragDropPromise = getCurrentWebview().onDragDropEvent(handleDragDrop);
    const stopCloseGuardPromise = quitGuard.listen();
    const stopHelpMenuPromise = getCurrentWindow().listen(HELP_REQUESTED_EVENT, () => {
      view = "help";
    });

    return () => {
      void stopDragDropPromise.then((stopListening) => stopListening());
      void stopCloseGuardPromise.then((stopListening) => stopListening());
      void stopHelpMenuPromise.then((stopListening) => stopListening());
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
      {platform}
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
  <DropOverlay isConverting={queue.isConverting} />
{/if}

<QuitDialog bind:open={quitGuard.isDialogOpen} isQuitting={quitGuard.isQuitting} onQuit={() => quitGuard.quit()} />
