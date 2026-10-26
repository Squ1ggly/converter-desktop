<script lang="ts">
  import ChevronLeftIcon from "@lucide/svelte/icons/chevron-left";
  import CircleHelpIcon from "@lucide/svelte/icons/circle-help";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import type { AppView } from "$lib/views";
  import IconAction from "./IconAction.svelte";
  import ShortcutHint from "./ShortcutHint.svelte";

  interface Props {
    view: AppView;
    platform: string;
    hasFiles: boolean;
    isConverting: boolean;
    onAddFiles: () => void;
    onClear: () => void;
    onToggleView: (view: AppView) => void;
    onBack: () => void;
  }

  let { view, platform, hasFiles, isConverting, onAddFiles, onClear, onToggleView, onBack }: Props = $props();

  const PAGE_TITLES: Record<AppView, string> = {
    converter: "Converter",
    settings: "Settings",
    help: "Help",
  };

  const isMac = $derived(platform === "macos");
  const pageTitle = $derived(PAGE_TITLES[view]);

  let titleHeading = $state<HTMLHeadingElement | null>(null);
  let previousView: AppView | null = null;

  // A new page replaces what had focus, so focus moves to the page's title.
  $effect(() => {
    const currentView = view;
    const hasChanged = previousView !== null && previousView !== currentView;
    previousView = currentView;
    if (!hasChanged) {
      return;
    }
    titleHeading?.focus({ preventScroll: true });
  });
  // Other platforms already show "Converter" in their native title bar.
  const showsInlineTitle = $derived(!isMac && view !== "converter");
</script>

<!-- On macOS this replaces the title bar, so it is also the window drag area. -->
<header
  class="bg-background relative flex h-13 shrink-0 items-center gap-2 border-b px-3 {isMac ? 'pl-21' : ''}"
  data-tauri-drag-region
>
  {#if isMac}
    <h1
      bind:this={titleHeading}
      tabindex="-1"
      class="text-muted-foreground pointer-events-none absolute inset-x-0 text-center text-[13px] font-normal outline-none"
      data-tauri-drag-region
    >
      {pageTitle}
    </h1>
  {:else}
    <h1 bind:this={titleHeading} tabindex="-1" class="outline-none {showsInlineTitle ? 'order-1 text-sm font-medium' : 'sr-only'}">
      {pageTitle}
    </h1>
  {/if}

  {#if view !== "converter"}
    <Button variant="ghost" size="sm" class="relative" onclick={onBack}>
      <ChevronLeftIcon />
      Back
    </Button>
  {/if}

  <div class="relative order-2 ml-auto flex items-center gap-1">
    {#if view === "converter" && hasFiles}
      <Button variant="ghost" size="sm" onclick={onClear} disabled={isConverting}>Clear</Button>
      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <Button {...props} variant="outline" size="sm" class="mr-2" onclick={onAddFiles} disabled={isConverting}>
              <PlusIcon />
              Add files
            </Button>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content>
          Add files
          <ShortcutHint shortcut="open" {platform} />
        </Tooltip.Content>
      </Tooltip.Root>
    {/if}
    <IconAction label="Settings" shortcut="settings" {platform} isPressed={view === "settings"} onclick={() => onToggleView("settings")}>
      <SettingsIcon />
    </IconAction>
    <IconAction label="Help" shortcut="help" {platform} isPressed={view === "help"} onclick={() => onToggleView("help")}>
      <CircleHelpIcon />
    </IconAction>
  </div>
</header>
