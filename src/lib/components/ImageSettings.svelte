<script lang="ts">
  import * as Field from "$lib/components/ui/field/index.js";
  import * as InputGroup from "$lib/components/ui/input-group/index.js";
  import { Slider } from "$lib/components/ui/slider/index.js";
  import type { FormatOption, ResizeOption } from "$lib/backend";
  import type { Preferences } from "$lib/preferences.svelte";
  import FormatPicker from "./FormatPicker.svelte";
  import OptionToggle from "./OptionToggle.svelte";

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
    { value: "fit", label: "Fit" },
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

  // Blank or invalid number fields mean "no limit".
  function parsePositiveWholeNumber(text: string): number | null {
    const parsedNumber = Number.parseInt(text, 10);
    const isUsable = Number.isFinite(parsedNumber) && parsedNumber >= 1;
    if (!isUsable) {
      return null;
    }
    return parsedNumber;
  }

  function setPercent(text: string): void {
    const percent = parsePositiveWholeNumber(text) ?? DEFAULT_SCALE_PERCENT;
    const clampedPercent = Math.min(MAX_SCALE_PERCENT, Math.max(MIN_SCALE_PERCENT, percent));
    preferences.resize = { mode: "percent", percent: clampedPercent };
  }

  function setFitSide(side: "maxWidth" | "maxHeight", text: string): void {
    const currentFit: FitResize =
      preferences.resize.mode === "fit" ? preferences.resize : { mode: "fit", maxWidth: null, maxHeight: null };
    const sideLimit = parsePositiveWholeNumber(text);

    const updatedFit: FitResize =
      side === "maxWidth" ? { ...currentFit, maxWidth: sideLimit } : { ...currentFit, maxHeight: sideLimit };
    preferences.resize = updatedFit;
  }
</script>

<Field.Group>
  <FormatPicker label="Format" options={formatOptions} bind:value={preferences.imageFormat} />

  {#if usesQuality}
    <Field.Field>
      <div class="flex items-center justify-between">
        <Field.Label for="image-quality">Quality</Field.Label>
        <span class="text-muted-foreground text-sm tabular-nums">{preferences.imageQuality}</span>
      </div>
      <Slider id="image-quality" type="single" min={1} max={100} step={1} bind:value={preferences.imageQuality} aria-label="Quality" />
      <div class="text-muted-foreground flex justify-between text-xs" aria-hidden="true">
        <span>Smaller</span>
        <span>Sharper</span>
      </div>
    </Field.Field>
  {/if}

  <OptionToggle
    label="Size"
    options={RESIZE_MODES}
    bind:value={() => preferences.resize.mode, (mode) => (preferences.resize = defaultResizeFor(mode))}
  />

  {#if preferences.resize.mode === "percent"}
    <Field.Field>
      <Field.Label for="resize-percent" class="sr-only">Scale</Field.Label>
      <InputGroup.Root class="w-32">
        <InputGroup.Input
          id="resize-percent"
          type="number"
          min={MIN_SCALE_PERCENT}
          max={MAX_SCALE_PERCENT}
          value={preferences.resize.percent}
          onchange={(event) => setPercent(event.currentTarget.value)}
        />
        <InputGroup.Addon align="inline-end">%</InputGroup.Addon>
      </InputGroup.Root>
    </Field.Field>
  {:else if preferences.resize.mode === "fit"}
    <Field.Field>
      <div class="flex items-center gap-2">
        <Field.Label for="resize-width" class="sr-only">Max width</Field.Label>
        <InputGroup.Root>
          <InputGroup.Input
            id="resize-width"
            type="number"
            min={1}
            placeholder="Width"
            value={preferences.resize.maxWidth ?? ""}
            onchange={(event) => setFitSide("maxWidth", event.currentTarget.value)}
          />
          <InputGroup.Addon align="inline-end">px</InputGroup.Addon>
        </InputGroup.Root>
        <span class="text-muted-foreground" aria-hidden="true">×</span>
        <Field.Label for="resize-height" class="sr-only">Max height</Field.Label>
        <InputGroup.Root>
          <InputGroup.Input
            id="resize-height"
            type="number"
            min={1}
            placeholder="Height"
            value={preferences.resize.maxHeight ?? ""}
            onchange={(event) => setFitSide("maxHeight", event.currentTarget.value)}
          />
          <InputGroup.Addon align="inline-end">px</InputGroup.Addon>
        </InputGroup.Root>
      </div>
      <Field.Description>Keeps proportions. Leave a side blank to scale by the other.</Field.Description>
    </Field.Field>
  {/if}
</Field.Group>
