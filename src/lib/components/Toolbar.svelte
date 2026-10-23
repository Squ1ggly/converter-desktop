<script lang="ts">
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$lib/components/ui/button/index.js";

  interface Props {
    isMac: boolean;
    hasFiles: boolean;
    isConverting: boolean;
    onAddFiles: () => void;
    onClear: () => void;
  }

  let { isMac, hasFiles, isConverting, onAddFiles, onClear }: Props = $props();
</script>

<!-- On macOS this replaces the title bar, so it doubles as the window drag area. -->
<header
  class="bg-background flex h-13 shrink-0 items-center justify-between border-b pr-3 {isMac ? 'pl-21' : 'pl-4'}"
  data-tauri-drag-region
>
  <h1 class="text-sm font-semibold" data-tauri-drag-region>Converter</h1>

  {#if hasFiles}
    <div class="flex gap-2">
      <Button variant="ghost" size="sm" onclick={onClear} disabled={isConverting}>Clear</Button>
      <Button variant="outline" size="sm" onclick={onAddFiles} disabled={isConverting}>
        <PlusIcon />
        Add files
      </Button>
    </div>
  {/if}
</header>
