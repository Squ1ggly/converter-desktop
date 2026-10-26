<script lang="ts">
  import * as Field from "$lib/components/ui/field/index.js";
  import type { FormatOption } from "$lib/backend";
  import { FRAME_RATE_OPTIONS, RESOLUTION_OPTIONS, VIDEO_QUALITY_OPTIONS } from "$lib/conversion-options";
  import type { Preferences } from "$lib/preferences";
  import FormatPicker from "./FormatPicker.svelte";
  import OptionSelect from "./OptionSelect.svelte";
  import OptionToggle from "./OptionToggle.svelte";

  interface Props {
    formatOptions: FormatOption[];
    preferences: Preferences;
  }

  let { formatOptions, preferences = $bindable() }: Props = $props();

  const isGif = $derived(preferences.videoFormat === "gif");
  const frameRateDescription = $derived(isGif && preferences.frameRate === null ? "GIFs default to 15 fps." : undefined);
</script>

<Field.Group>
  <FormatPicker label="Format" options={formatOptions} bind:value={preferences.videoFormat} />

  {#if !isGif}
    <OptionToggle label="Quality" options={VIDEO_QUALITY_OPTIONS} bind:value={preferences.videoQuality} />
  {/if}

  <OptionSelect
    id="video-resolution"
    label="Resolution"
    options={RESOLUTION_OPTIONS}
    bind:value={preferences.maxHeight}
    description="Only scales down."
  />

  <OptionSelect
    id="video-frame-rate"
    label="Frame rate"
    options={FRAME_RATE_OPTIONS}
    bind:value={preferences.frameRate}
    description={frameRateDescription}
  />
</Field.Group>
