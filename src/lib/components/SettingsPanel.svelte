<script lang="ts">
  import type { ConversionCatalog } from "../lib/backend";
  import type { Preferences } from "../lib/preferences.svelte";
  import ConvertBar from "./ConvertBar.svelte";
  import ImageSettings from "./ImageSettings.svelte";
  import SaveLocation from "./SaveLocation.svelte";
  import VideoSettings from "./VideoSettings.svelte";

  interface Props {
    catalog: ConversionCatalog;
    preferences: Preferences;
    hasImages: boolean;
    hasVideos: boolean;
    fileCount: number;
    finishedCount: number;
    isConverting: boolean;
    isCancelling: boolean;
    isRerun: boolean;
    shortcutLabel: string;
    onConvert: () => void;
    onCancel: () => void;
  }

  let {
    catalog,
    preferences = $bindable(),
    hasImages,
    hasVideos,
    fileCount,
    finishedCount,
    isConverting,
    isCancelling,
    isRerun,
    shortcutLabel,
    onConvert,
    onCancel,
  }: Props = $props();
</script>

<aside class="settings-panel" aria-label="Conversion settings">
  <!-- Settings can't change mid-conversion; a disabled fieldset locks every
       control inside it at once. -->
  <fieldset class="settings-scroll" disabled={isConverting}>
    {#if hasImages}
      <ImageSettings formatOptions={catalog.imageOutputs} bind:preferences />
    {/if}
    {#if hasVideos}
      <VideoSettings formatOptions={catalog.videoOutputs} bind:preferences />
    {/if}
    <section class="settings-section">
      <SaveLocation bind:saveLocation={preferences.saveLocation} />
    </section>
  </fieldset>

  <ConvertBar {fileCount} {finishedCount} {isConverting} {isCancelling} {isRerun} {shortcutLabel} {onConvert} {onCancel} />
</aside>

<style>
  .settings-panel {
    display: grid;
    grid-template-rows: 1fr auto;
    min-height: 0;
    border-left: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  .settings-scroll {
    min-height: 0;
    min-width: 0;
    overflow-y: auto;
    border: none;
  }

  .settings-scroll :global(.settings-section) {
    display: grid;
    gap: 18px;
    padding: 20px;
  }

  .settings-scroll :global(.settings-section + .settings-section) {
    border-top: 1px solid var(--color-border);
  }

  .settings-scroll :global(.section-title) {
    font-size: var(--font-size-title);
    font-weight: 600;
  }
</style>
