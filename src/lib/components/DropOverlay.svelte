<script lang="ts">
  import FolderInputIcon from "@lucide/svelte/icons/folder-input";
  import HourglassIcon from "@lucide/svelte/icons/hourglass";

  interface Props {
    // Drops are refused during a conversion, so the overlay says so.
    isConverting: boolean;
  }

  let { isConverting }: Props = $props();
</script>

<div class="bg-background/85 pointer-events-none fixed inset-0 z-50 grid p-4 backdrop-blur-sm" aria-hidden="true">
  <div class="text-foreground grid place-content-center justify-items-center gap-3 rounded-xl border-2 border-dashed text-lg font-medium">
    {#if isConverting}
      <HourglassIcon class="size-8" />
      Wait for the conversion to finish
    {:else}
      <FolderInputIcon class="size-8" />
      Drop to add
    {/if}
  </div>
</div>
