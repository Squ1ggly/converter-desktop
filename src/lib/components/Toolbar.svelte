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

<!-- On macOS this replaces the title bar, so it shows the title and is the drag
     area. Other platforms keep their native title bar. -->
{#if isMac || hasFiles}
  <header class="bg-background relative flex h-13 shrink-0 items-center justify-end border-b px-3" data-tauri-drag-region>
    {#if isMac}
      <h1
        class="text-muted-foreground pointer-events-none absolute inset-x-0 text-center text-[13px] font-normal"
        data-tauri-drag-region
      >
        Converter
      </h1>
    {/if}

    {#if hasFiles}
      <div class="relative flex gap-2">
        <Button variant="ghost" size="sm" onclick={onClear} disabled={isConverting}>Clear</Button>
        <Button variant="outline" size="sm" onclick={onAddFiles} disabled={isConverting}>
          <PlusIcon />
          Add files
        </Button>
      </div>
    {/if}
  </header>
{/if}
