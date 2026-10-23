<script lang="ts">
  import { ArrowRight, Ban, CircleAlert, CircleCheck, ExternalLink, FileImage, FilePlay, FolderSearch, X } from "@lucide/svelte";
  import { extensionOf, formatBytes } from "../lib/formatting";
  import type { QueueItem } from "../lib/queue.svelte";

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

<li class="file-row" class:file-row--failed={item.status === "failed"}>
  <div class="thumbnail" aria-hidden="true">
    {#if showsPreview}
      <img src={item.previewUrl} alt="" loading="lazy" decoding="async" onerror={() => (hasPreviewFailed = true)} />
    {:else if item.kind === "image"}
      <FileImage size={20} />
    {:else}
      <FilePlay size={20} />
    {/if}
  </div>

  <div class="details">
    <p class="file-name selectable" title={item.path}>{item.name}</p>
    {#if item.status === "failed" && item.errorMessage}
      <p class="error-message selectable" role="alert">{item.errorMessage}</p>
    {:else}
      <p class="meta">
        <span>{formatBytes(item.sizeBytes)}</span>
        <span class="conversion-label">
          {sourceLabel}
          <ArrowRight size={11} aria-label="to" />
          {targetLabel}
        </span>
      </p>
    {/if}
  </div>

  <div class="status">
    {#if item.status === "ready"}
      <button type="button" class="icon-button" onclick={() => onRemove(item.id)} disabled={isLocked} aria-label="Remove {item.name}" title="Remove">
        <X size={16} />
      </button>
    {:else if item.status === "waiting"}
      <span class="status-text">Waiting</span>
    {:else if item.status === "converting"}
      {#if item.progressPercent === null}
        <div class="progress-track progress-track--indeterminate" role="progressbar" aria-label="Converting {item.name}">
          <div class="progress-fill"></div>
        </div>
      {:else}
        <div class="progress-track" role="progressbar" aria-label="Converting {item.name}" aria-valuenow={progressPercent} aria-valuemin={0} aria-valuemax={100}>
          <div class="progress-fill" style:width="{progressPercent}%"></div>
        </div>
        <span class="status-text percent">{progressPercent}%</span>
      {/if}
    {:else if item.status === "done" && item.outputPath}
      {@const outputPath = item.outputPath}
      <CircleCheck class="done-icon" size={16} aria-label="Converted" />
      <button type="button" class="icon-button" onclick={() => onOpen(outputPath)} aria-label="Open converted {item.name}" title="Open">
        <ExternalLink size={16} />
      </button>
      <button type="button" class="icon-button" onclick={() => onReveal(outputPath)} aria-label="{revealLabel}: {item.name}" title={revealLabel}>
        <FolderSearch size={16} />
      </button>
    {:else if item.status === "failed"}
      <span class="status-text status-text--failed">
        <CircleAlert size={14} aria-hidden="true" />
        Failed
      </span>
    {:else if item.status === "cancelled"}
      <span class="status-text">
        <Ban size={14} aria-hidden="true" />
        Cancelled
      </span>
    {/if}
  </div>
</li>

<style>
  .file-row {
    display: grid;
    grid-template-columns: 40px 1fr auto;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-radius: var(--radius-medium);
  }

  .file-row:hover {
    background: var(--color-surface-hover);
  }

  .file-row--failed {
    background: var(--color-danger-subtle);
  }

  .thumbnail {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: var(--radius-small);
    background: var(--color-surface-sunken);
    color: var(--color-text-tertiary);
    overflow: hidden;
  }

  .thumbnail img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .details {
    min-width: 0;
  }

  .file-name {
    font-weight: 500;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    display: flex;
    gap: 10px;
    font-size: var(--font-size-small);
    color: var(--color-text-tertiary);
    font-variant-numeric: tabular-nums;
  }

  .conversion-label {
    display: inline-flex;
    align-items: center;
    gap: 3px;
  }

  .error-message {
    font-size: var(--font-size-small);
    color: var(--color-danger);
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    line-clamp: 2;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
  }

  .status {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    min-width: 96px;
  }

  .status-text {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: var(--font-size-small);
    color: var(--color-text-tertiary);
  }

  .status-text--failed {
    color: var(--color-danger);
    font-weight: 500;
  }

  .percent {
    width: 34px;
    justify-content: flex-end;
    font-variant-numeric: tabular-nums;
  }

  .status :global(.done-icon) {
    color: var(--color-success);
    margin-right: 2px;
  }

  .progress-track {
    width: 88px;
    height: 4px;
    border-radius: 2px;
    background: var(--color-surface-sunken);
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    border-radius: inherit;
    background: var(--color-accent);
    transition: width 200ms var(--easing-standard);
  }

  .progress-track--indeterminate .progress-fill {
    width: 40%;
    animation: slide 1.1s ease-in-out infinite;
  }

  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(250%);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .progress-track--indeterminate .progress-fill {
      animation: none;
      width: 100%;
      opacity: 0.5;
    }
  }
</style>
