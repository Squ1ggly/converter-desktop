<script lang="ts">
  import ArrowRightIcon from "@lucide/svelte/icons/arrow-right";
  import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
  import FileImageIcon from "@lucide/svelte/icons/file-image";
  import FilePlayIcon from "@lucide/svelte/icons/file-play";
  import FolderSearchIcon from "@lucide/svelte/icons/folder-search";
  import XIcon from "@lucide/svelte/icons/x";
  import { Badge } from "$lib/components/ui/badge/index.js";
  import * as Item from "$lib/components/ui/item/index.js";
  import { Progress } from "$lib/components/ui/progress/index.js";
  import { Spinner } from "$lib/components/ui/spinner/index.js";
  import { extensionOf, formatBytes } from "$lib/formatting";
  import type { QueueItem } from "$lib/queue.svelte";
  import IconAction from "./IconAction.svelte";

  interface Props {
    item: QueueItem;
    targetLabel: string;
    revealLabel: string;
    isLocked: boolean;
    onRemove: (itemId: string) => void;
    onOpen: (outputPath: string) => void;
    onReveal: (outputPath: string) => void;
  }

  let { item, targetLabel, revealLabel, isLocked, onRemove, onOpen, onReveal }: Props = $props();

  let hasPreviewFailed = $state(false);

  const sourceLabel = $derived(extensionOf(item.name) || "File");
  const showsPreview = $derived(item.previewUrl !== null && !hasPreviewFailed);
  const progressPercent = $derived(Math.round(item.progressPercent ?? 0));
</script>

<Item.Root role="listitem" size="sm" class={item.status === "failed" ? "bg-destructive/5" : ""} data-testid="file-row" data-status={item.status}>
  <Item.Media variant="image" class="bg-muted text-muted-foreground">
    {#if showsPreview}
      <img src={item.previewUrl} alt="" loading="lazy" decoding="async" onerror={() => (hasPreviewFailed = true)} />
    {:else if item.kind === "image"}
      <FileImageIcon class="size-5" />
    {:else}
      <FilePlayIcon class="size-5" />
    {/if}
  </Item.Media>

  <Item.Content class="min-w-0">
    <Item.Title class="block w-full truncate select-text" title={item.path}>{item.name}</Item.Title>
    {#if item.status === "failed" && item.errorMessage}
      <Item.Description class="text-destructive select-text" role="alert">{item.errorMessage}</Item.Description>
    {:else}
      <Item.Description class="flex items-center gap-2 tabular-nums">
        {formatBytes(item.sizeBytes)}
        <span class="inline-flex items-center gap-1">
          {sourceLabel}
          <ArrowRightIcon class="size-3" aria-label="to" />
          {targetLabel}
        </span>
      </Item.Description>
    {/if}
  </Item.Content>

  <Item.Actions class="min-w-28 justify-end">
    {#if item.status === "ready"}
      <IconAction label="Remove" disabled={isLocked} onclick={() => onRemove(item.id)}>
        <XIcon />
      </IconAction>
    {:else if item.status === "waiting"}
      <span class="text-muted-foreground text-xs">Queued</span>
    {:else if item.status === "converting" && item.progressPercent === null}
      <Spinner aria-label="Converting" />
    {:else if item.status === "converting"}
      <Progress value={progressPercent} class="w-20" aria-label="Converting" />
      <span class="text-muted-foreground w-9 text-right text-xs tabular-nums">{progressPercent}%</span>
    {:else if item.status === "done" && item.outputPath}
      {@const outputPath = item.outputPath}
      <Badge variant="secondary">Done</Badge>
      <IconAction label="Open" onclick={() => onOpen(outputPath)}>
        <ExternalLinkIcon />
      </IconAction>
      <IconAction label={revealLabel} onclick={() => onReveal(outputPath)}>
        <FolderSearchIcon />
      </IconAction>
    {:else if item.status === "failed"}
      <Badge variant="destructive">Failed</Badge>
    {:else if item.status === "cancelled"}
      <Badge variant="secondary">Cancelled</Badge>
    {/if}
  </Item.Actions>
</Item.Root>
