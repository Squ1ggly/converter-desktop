<script lang="ts">
  import { untrack } from "svelte";
  import type { Tooltip as TooltipPrimitive } from "bits-ui";
  import { createVirtualizer } from "@tanstack/svelte-virtual";
  import type { QueueItem } from "$lib/queue.svelte";
  import FileRow from "./FileRow.svelte";

  interface Props {
    title: string;
    showsTitle: boolean;
    items: QueueItem[];
    targetLabel: string;
    revealLabel: string;
    isLocked: boolean;
    // The scrolling viewport the list sits in, and the content that grows inside it.
    scrollElement: HTMLElement;
    scrollContent: HTMLElement;
    rowTooltip: TooltipPrimitive.Tether<string>;
    onRemove: (itemId: string) => void;
    onOpen: (outputPath: string) => void;
    onReveal: (outputPath: string) => void;
  }

  let {
    title,
    showsTitle,
    items,
    targetLabel,
    revealLabel,
    isLocked,
    scrollElement,
    scrollContent,
    rowTooltip,
    onRemove,
    onOpen,
    onReveal,
  }: Props = $props();

  // A row with a one-line description. Taller rows (long errors) are measured.
  const ESTIMATED_ROW_HEIGHT_PX = 66;
  const ROW_GAP_PX = 4;
  // Rows rendered beyond the visible ones, so scrolling and tabbing stay smooth.
  const OVERSCAN_ROW_COUNT = 8;

  let listElement = $state<HTMLElement | null>(null);
  // Where this list starts inside the scrolling content; the other group may sit above it.
  let listOffsetPx = $state(0);

  const virtualizer = createVirtualizer<HTMLElement, HTMLElement>({
    count: 0,
    getScrollElement: () => scrollElement,
    estimateSize: () => ESTIMATED_ROW_HEIGHT_PX,
    gap: ROW_GAP_PX,
    overscan: OVERSCAN_ROW_COUNT,
    // Measuring a row can resize the list; waiting a frame avoids ResizeObserver loop errors.
    useAnimationFrameWithResizeObserver: true,
  });

  $effect(() => {
    const options = {
      count: items.length,
      scrollMargin: listOffsetPx,
      getItemKey: (index: number) => items[index]?.id ?? index,
    };
    untrack(() => $virtualizer.setOptions(options));
  });

  function measureListOffset(): void {
    if (!listElement) {
      return;
    }
    const listTop = listElement.getBoundingClientRect().top;
    const contentTop = scrollContent.getBoundingClientRect().top;
    listOffsetPx = listTop - contentTop;
  }

  $effect(() => {
    let pendingFrame = 0;
    const resizeObserver = new ResizeObserver(() => {
      cancelAnimationFrame(pendingFrame);
      pendingFrame = requestAnimationFrame(measureListOffset);
    });
    resizeObserver.observe(scrollContent);
    measureListOffset();
    return () => {
      cancelAnimationFrame(pendingFrame);
      resizeObserver.disconnect();
    };
  });

  function measureRow(rowElement: HTMLElement): void {
    $virtualizer.measureElement(rowElement);
  }
</script>

<div role="group" aria-label={title}>
  {#if showsTitle}
    <h3 class="text-muted-foreground px-3 pt-1 pb-2 text-xs font-medium">
      {title} · {items.length}
    </h3>
  {/if}
  <div bind:this={listElement} role="list" class="relative w-full" style:height="{$virtualizer.getTotalSize()}px">
    {#each $virtualizer.getVirtualItems() as row (row.key)}
      {@const item = items[row.index]}
      {#if item}
        <div
          role="listitem"
          aria-setsize={items.length}
          aria-posinset={row.index + 1}
          data-index={row.index}
          class="absolute inset-x-0 top-0"
          style:transform="translateY({row.start - listOffsetPx}px)"
          {@attach measureRow}
        >
          <FileRow {item} {targetLabel} {revealLabel} {isLocked} {rowTooltip} {onRemove} {onOpen} {onReveal} />
        </div>
      {/if}
    {/each}
  </div>
</div>
