<script lang="ts">
  import { ArrowRight, LoaderCircle } from "@lucide/svelte";
  import { pluralise } from "../lib/formatting";

  interface Props {
    fileCount: number;
    finishedCount: number;
    isConverting: boolean;
    isCancelling: boolean;
    // True once every file has a result; converting again redoes them all.
    isRerun: boolean;
    shortcutLabel: string;
    onConvert: () => void;
    onCancel: () => void;
  }

  let { fileCount, finishedCount, isConverting, isCancelling, isRerun, shortcutLabel, onConvert, onCancel }: Props = $props();

  const convertLabel = $derived(isRerun ? "Convert again" : `Convert ${pluralise(fileCount, "file")}`);

  const overallPercent = $derived(fileCount === 0 ? 0 : Math.round((finishedCount / fileCount) * 100));
</script>

<div class="convert-bar">
  {#if isConverting}
    <div class="progress-summary" aria-live="polite">
      <LoaderCircle class="spinner" size={16} aria-hidden="true" />
      <span>Converting {Math.min(finishedCount + 1, fileCount)} of {fileCount}…</span>
    </div>
    <div class="overall-track" role="progressbar" aria-label="Overall progress" aria-valuenow={overallPercent} aria-valuemin={0} aria-valuemax={100}>
      <div class="overall-fill" style:width="{overallPercent}%"></div>
    </div>
    <button type="button" class="button button--large" onclick={onCancel} disabled={isCancelling}>
      {isCancelling ? "Cancelling…" : "Cancel"}
    </button>
  {:else}
    <button type="button" class="button button--primary button--large" onclick={onConvert} disabled={fileCount === 0}>
      {convertLabel}
      <ArrowRight size={16} aria-hidden="true" />
    </button>
    <p class="shortcut-hint">or press {shortcutLabel}↩</p>
  {/if}
</div>

<style>
  .convert-bar {
    display: grid;
    gap: 10px;
    padding: 16px 20px 20px;
    border-top: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .button--large {
    width: 100%;
  }

  .shortcut-hint {
    text-align: center;
    color: var(--color-text-tertiary);
  }

  .progress-summary {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 500;
  }

  .overall-track {
    height: 6px;
    border-radius: 3px;
    background: var(--color-surface-sunken);
    overflow: hidden;
  }

  .overall-fill {
    height: 100%;
    border-radius: inherit;
    background: var(--color-accent);
    transition: width 300ms var(--easing-standard);
  }

  .progress-summary :global(.spinner) {
    color: var(--color-accent);
    animation: spin 0.9s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
