<script lang="ts">
  import FolderSearchIcon from "@lucide/svelte/icons/folder-search";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Spinner } from "$lib/components/ui/spinner/index.js";
  import { Tooltip as TooltipPrimitive } from "bits-ui";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { pluralise } from "$lib/formatting";
  import type { FileQueue, QueueItem } from "$lib/queue.svelte";
  import FileGroupList from "./FileGroupList.svelte";

  interface Props {
    queue: FileQueue;
    isAdding: boolean;
    imageTargetLabel: string;
    videoTargetLabel: string;
    revealLabel: string;
    onRemove: (itemId: string) => void;
    onOpen: (outputPath: string) => void;
    onReveal: (outputPath: string) => void;
  }

  let { queue, isAdding, imageTargetLabel, videoTargetLabel, revealLabel, onRemove, onOpen, onReveal }: Props = $props();

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

  let scrollElement = $state<HTMLElement | null>(null);
  let scrollContent = $state<HTMLElement | null>(null);
  const rowTooltip = TooltipPrimitive.createTether<string>();
  const firstOutputPath = $derived(queue.items.find((item) => item.outputPath !== null)?.outputPath ?? null);
  const summary = $derived(queue.summary);

  // Before a run: "3 files" over "2 images · 1 video". After: what happened,
  // led by the converted count, or by the failures if nothing converted.
  const resultParts = $derived.by(() => {
    const parts: string[] = [];
    if (summary.doneCount > 0) {
      parts.push(`${pluralise(summary.doneCount, "file")} converted`);
    }
    if (summary.failedCount > 0) {
      parts.push(`${summary.failedCount} failed`);
    }
    if (summary.cancelledCount > 0) {
      parts.push(`${summary.cancelledCount} cancelled`);
    }
    return parts;
  });

  const kindParts = $derived.by(() => {
    const parts: string[] = [];
    if (queue.images.length > 0) {
      parts.push(pluralise(queue.images.length, "image"));
    }
    if (queue.videos.length > 0) {
      parts.push(pluralise(queue.videos.length, "video"));
    }
    return parts;
  });

  const heading = $derived(queue.hasFinishedRun ? (resultParts[0] ?? "") : pluralise(queue.items.length, "file"));
  const subheading = $derived((queue.hasFinishedRun ? resultParts.slice(1) : kindParts).join(" · "));
</script>

<section class="flex min-h-0 min-w-0 flex-col" aria-label="Files">
  <header class="flex min-h-16 items-center justify-between gap-3 border-b px-5 py-3">
    <div aria-live="polite">
      <h2 class="text-sm font-semibold" data-testid="list-heading">{heading}</h2>
      {#if isAdding}
        <p class="text-muted-foreground flex items-center gap-1.5 text-xs">
          <Spinner class="size-3" role="presentation" aria-label={undefined} aria-hidden="true" />
          Adding files
        </p>
      {:else}
        <p class="text-muted-foreground text-xs">{subheading}</p>
      {/if}
    </div>
    {#if queue.hasFinishedRun && firstOutputPath}
      {@const outputPath = firstOutputPath}
      <Button variant="outline" size="sm" onclick={() => onReveal(outputPath)}>
        <FolderSearchIcon />
        {revealLabel}
      </Button>
    {/if}
  </header>

  <ScrollArea class="min-h-0 flex-1" bind:viewportRef={scrollElement}>
    <div bind:this={scrollContent} class="flex flex-col gap-4 p-3">
      {#if scrollElement && scrollContent}
        {#each groups as group (group.title)}
          <FileGroupList
            title={group.title}
            showsTitle={showsGroupTitles}
            items={group.items}
            targetLabel={group.targetLabel}
            {revealLabel}
            isLocked={queue.isConverting}
            {scrollElement}
            {scrollContent}
            {rowTooltip}
            {onRemove}
            {onOpen}
            {onReveal}
          />
        {/each}
      {/if}
    </div>
  </ScrollArea>

  <Tooltip.Root tether={rowTooltip}>
    {#snippet children({ payload })}
      <Tooltip.Content>{payload}</Tooltip.Content>
    {/snippet}
  </Tooltip.Root>
</section>
