<script lang="ts">
  import FolderSearchIcon from "@lucide/svelte/icons/folder-search";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Item from "$lib/components/ui/item/index.js";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { pluralise } from "$lib/formatting";
  import type { FileQueue, QueueItem } from "$lib/queue.svelte";
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

  const heading = $derived(
    queue.hasFinishedRun ? `${pluralise(summary.doneCount, "file")} converted` : pluralise(queue.items.length, "file"),
  );

  const subheading = $derived.by(() => {
    const parts: string[] = [];
    if (queue.hasFinishedRun && summary.failedCount > 0) {
      parts.push(`${summary.failedCount} failed`);
    }
    if (queue.hasFinishedRun && summary.cancelledCount > 0) {
      parts.push(`${summary.cancelledCount} cancelled`);
    }
    if (!queue.hasFinishedRun && queue.images.length > 0) {
      parts.push(pluralise(queue.images.length, "image"));
    }
    if (!queue.hasFinishedRun && queue.videos.length > 0) {
      parts.push(pluralise(queue.videos.length, "video"));
    }

    const joinedParts = parts.join(" · ");
    return joinedParts;
  });
</script>

<section class="flex min-h-0 min-w-0 flex-col" aria-label="Files">
  <header class="flex min-h-16 items-center justify-between gap-3 border-b px-5 py-3">
    <div aria-live="polite">
      <h2 class="text-sm font-semibold" data-testid="list-heading">{heading}</h2>
      <p class="text-muted-foreground text-xs">{subheading}</p>
    </div>
    {#if queue.hasFinishedRun && firstOutputPath}
      {@const outputPath = firstOutputPath}
      <Button variant="outline" size="sm" onclick={() => onReveal(outputPath)}>
        <FolderSearchIcon />
        {revealLabel}
      </Button>
    {/if}
  </header>

  <ScrollArea class="min-h-0 flex-1">
    <div class="flex flex-col gap-4 p-3">
      {#each groups as group (group.title)}
        <div role="group" aria-label={group.title}>
          {#if showsGroupTitles}
            <h3 class="text-muted-foreground px-3 pt-1 pb-2 text-xs font-medium">
              {group.title} · {group.items.length}
            </h3>
          {/if}
          <Item.Group class="gap-1">
            {#each group.items as item (item.id)}
              <FileRow {item} targetLabel={group.targetLabel} {revealLabel} isLocked={queue.isConverting} {onRemove} {onOpen} {onReveal} />
            {/each}
          </Item.Group>
        </div>
      {/each}
    </div>
  </ScrollArea>
</section>
