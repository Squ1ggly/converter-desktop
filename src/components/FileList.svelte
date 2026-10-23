<script lang="ts">
  import { FolderSearch } from "@lucide/svelte";
  import { pluralise } from "../lib/formatting";
  import type { FileQueue, QueueItem } from "../lib/queue.svelte";
  import FileRow from "./FileRow.svelte";

  interface Props {
    queue: FileQueue;
    imageTargetLabel: string;
    videoTargetLabel: string;
    revealLabel: string;
    onRemove: (itemId: string) => void;
    onOpen: (outputPath: string) => void;
    onReveal: (outputPath: string) => void;
  }

  let { queue, imageTargetLabel, videoTargetLabel, revealLabel, onRemove, onOpen, onReveal }: Props = $props();

  interface FileGroup {
    title: string;
    items: QueueItem[];
    targetLabel: string;
  }

  const groups = $derived<FileGroup[]>(
    [
      { title: "Images", items: queue.images, targetLabel: imageTargetLabel },
      { title: "Videos", items: queue.videos, targetLabel: videoTargetLabel },
    ].filter((group) => group.items.length > 0),
  );
  const showsGroupTitles = $derived(groups.length > 1);
  const firstOutputPath = $derived(queue.items.find((item) => item.outputPath !== null)?.outputPath ?? null);
  const summary = $derived(queue.summary);

  // e.g. "3 images · 2 videos"
  const kindBreakdown = $derived.by(() => {
    const parts: string[] = [];
    if (queue.images.length > 0) {
      parts.push(pluralise(queue.images.length, "image"));
    }
    if (queue.videos.length > 0) {
      parts.push(pluralise(queue.videos.length, "video"));
    }

    const breakdown = parts.join(" · ");
    return breakdown;
  });
</script>

<section class="file-list" aria-label="Files to convert">
  <header class="list-header">
    {#if queue.hasFinishedRun}
      <div aria-live="polite">
        <h2 class="list-title">{pluralise(summary.doneCount, "file")} converted</h2>
        {#if summary.failedCount > 0 || summary.cancelledCount > 0}
          <p class="list-subtitle">
            {#if summary.failedCount > 0}<span class="failed-count">{summary.failedCount} failed</span>{/if}
            {#if summary.failedCount > 0 && summary.cancelledCount > 0}·{/if}
            {#if summary.cancelledCount > 0}{summary.cancelledCount} cancelled{/if}
          </p>
        {/if}
      </div>
      {#if firstOutputPath}
        {@const outputPath = firstOutputPath}
        <button type="button" class="button" onclick={() => onReveal(outputPath)}>
          <FolderSearch size={14} aria-hidden="true" />
          {revealLabel}
        </button>
      {/if}
    {:else}
      <div>
        <h2 class="list-title">{pluralise(queue.items.length, "file")}</h2>
        <p class="list-subtitle">{kindBreakdown}</p>
      </div>
    {/if}
  </header>

  <div class="list-scroll">
    {#each groups as group (group.title)}
      <div class="group" role="group" aria-label={group.title}>
        {#if showsGroupTitles}
          <h3 class="group-title">{group.title} <span class="group-count">{group.items.length}</span></h3>
        {/if}
        <ul class="rows">
          {#each group.items as item (item.id)}
            <FileRow {item} targetLabel={group.targetLabel} {revealLabel} isLocked={queue.isConverting} {onRemove} {onOpen} {onReveal} />
          {/each}
        </ul>
      </div>
    {/each}
  </div>
</section>

<style>
  .file-list {
    display: grid;
    grid-template-rows: auto 1fr;
    min-height: 0;
    min-width: 0;
  }

  .list-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 64px;
    padding: 12px 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .list-title {
    font-size: var(--font-size-title);
    font-weight: 600;
  }

  .list-subtitle {
    color: var(--color-text-secondary);
    font-size: var(--font-size-small);
  }

  .failed-count {
    color: var(--color-danger);
    font-weight: 500;
  }

  .list-scroll {
    min-height: 0;
    overflow-y: auto;
    padding: 8px 12px 24px;
  }

  .group + .group {
    margin-top: 12px;
  }

  .group-title {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 12px 4px;
    font-size: var(--font-size-small);
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .group-count {
    padding: 0 6px;
    border-radius: 999px;
    background: var(--color-surface-sunken);
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }

  .rows {
    list-style: none;
  }
</style>
