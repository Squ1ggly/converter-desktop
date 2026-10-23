<script lang="ts">
  import type { FormatOption, ResizeOption } from "../lib/backend";
  import type { Preferences } from "../lib/preferences.svelte";
  import FormatPicker from "./FormatPicker.svelte";
  import SegmentedControl from "./SegmentedControl.svelte";

  interface Props {
    formatOptions: FormatOption[];
    preferences: Preferences;
  }

  let { formatOptions, preferences = $bindable() }: Props = $props();

  type ResizeMode = ResizeOption["mode"];
  type FitResize = Extract<ResizeOption, { mode: "fit" }>;

  const RESIZE_MODES: { value: ResizeMode; label: string }[] = [
    { value: "original", label: "Original" },
    { value: "percent", label: "Scale" },
    { value: "fit", label: "Fit within" },
  ];
  const DEFAULT_SCALE_PERCENT = 50;
  const MIN_SCALE_PERCENT = 1;
  const MAX_SCALE_PERCENT = 1000;

  const selectedFormat = $derived(formatOptions.find((option) => option.id === preferences.imageFormat));
  const usesQuality = $derived(selectedFormat?.usesQuality ?? false);

  function defaultResizeFor(mode: ResizeMode): ResizeOption {
    if (mode === "percent") {
      return { mode, percent: DEFAULT_SCALE_PERCENT };
    }
    if (mode === "fit") {
      return { mode, maxWidth: null, maxHeight: null };
    }
    return { mode: "original" };
  }

  // Empty or invalid number fields mean "no limit".
  function toPositiveWholeNumber(value: number | null | undefined): number | null {
    const isUsable = typeof value === "number" && Number.isFinite(value) && value >= 1;
    if (!isUsable) {
      return null;
    }

    const wholeNumber = Math.round(value);
    return wholeNumber;
  }

  function setPercent(value: number | null): void {
    const percent = toPositiveWholeNumber(value) ?? DEFAULT_SCALE_PERCENT;
    const clampedPercent = Math.min(MAX_SCALE_PERCENT, Math.max(MIN_SCALE_PERCENT, percent));
    preferences.resize = { mode: "percent", percent: clampedPercent };
  }

  function setFitSide(side: "maxWidth" | "maxHeight", value: number | null): void {
    const currentFit: FitResize =
      preferences.resize.mode === "fit" ? preferences.resize : { mode: "fit", maxWidth: null, maxHeight: null };
    const sideLimit = toPositiveWholeNumber(value);

    const updatedFit: FitResize =
      side === "maxWidth" ? { ...currentFit, maxWidth: sideLimit } : { ...currentFit, maxHeight: sideLimit };
    preferences.resize = updatedFit;
  }
</script>

<section class="settings-section" aria-labelledby="image-settings-title">
  <h2 id="image-settings-title" class="section-title">Images</h2>

  <FormatPicker legend="Convert to" name="image-format" options={formatOptions} bind:value={preferences.imageFormat} />

  {#if usesQuality}
    <div class="field">
      <div class="quality-header">
        <label class="field-label" for="image-quality">Quality</label>
        <output class="quality-value" for="image-quality">{preferences.imageQuality}</output>
      </div>
      <input id="image-quality" type="range" min="1" max="100" bind:value={preferences.imageQuality} />
      <div class="range-ends field-help" aria-hidden="true">
        <span>Smaller file</span>
        <span>Better quality</span>
      </div>
    </div>
  {/if}

  <SegmentedControl
    legend="Size"
    name="resize-mode"
    options={RESIZE_MODES}
    bind:value={() => preferences.resize.mode, (mode) => (preferences.resize = defaultResizeFor(mode))}
  />

  {#if preferences.resize.mode === "percent"}
    <div class="inline-inputs">
      <label class="visually-hidden" for="resize-percent">Scale percentage</label>
      <input
        id="resize-percent"
        class="text-input number-input"
        type="number"
        min={MIN_SCALE_PERCENT}
        max={MAX_SCALE_PERCENT}
        bind:value={() => (preferences.resize.mode === "percent" ? preferences.resize.percent : null), setPercent}
      />
      <span class="unit">% of the original size</span>
    </div>
  {:else if preferences.resize.mode === "fit"}
    <div class="inline-inputs">
      <label class="visually-hidden" for="resize-width">Maximum width in pixels</label>
      <input
        id="resize-width"
        class="text-input number-input"
        type="number"
        min="1"
        placeholder="Width"
        bind:value={
          () => (preferences.resize.mode === "fit" ? preferences.resize.maxWidth : null),
          (value) => setFitSide("maxWidth", value)
        }
      />
      <span class="unit" aria-hidden="true">×</span>
      <label class="visually-hidden" for="resize-height">Maximum height in pixels</label>
      <input
        id="resize-height"
        class="text-input number-input"
        type="number"
        min="1"
        placeholder="Height"
        bind:value={
          () => (preferences.resize.mode === "fit" ? preferences.resize.maxHeight : null),
          (value) => setFitSide("maxHeight", value)
        }
      />
      <span class="unit">px</span>
    </div>
    <p class="field-help">Keeps the proportions. Leave a side blank to let it follow the other.</p>
  {/if}
</section>

<style>
  .quality-header {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
  }

  .quality-value {
    font-variant-numeric: tabular-nums;
    font-weight: 600;
  }

  input[type="range"] {
    width: 100%;
    accent-color: var(--color-accent);
  }

  .range-ends {
    display: flex;
    justify-content: space-between;
    margin-top: -4px;
  }

  .inline-inputs {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: -6px;
  }

  .number-input {
    width: 84px;
    font-variant-numeric: tabular-nums;
  }

  .unit {
    color: var(--color-text-secondary);
  }
</style>
