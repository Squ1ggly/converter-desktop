<script lang="ts">
  import * as Field from "$lib/components/ui/field/index.js";
  import type { FormatOption, VideoQuality } from "$lib/backend";
  import type { Preferences } from "$lib/preferences.svelte";
  import FormatPicker from "./FormatPicker.svelte";
  import OptionSelect from "./OptionSelect.svelte";
  import OptionToggle from "./OptionToggle.svelte";

  interface Props {
    formatOptions: FormatOption[];
    preferences: Preferences;
  }

  let { formatOptions, preferences = $bindable() }: Props = $props();

  const QUALITY_OPTIONS: { value: VideoQuality; label: string }[] = [
    { value: "high", label: "High" },
    { value: "balanced", label: "Balanced" },
    { value: "small", label: "Small" },
  ];

  const RESOLUTION_OPTIONS = [
    { value: null, label: "Original" },
    { value: 2160, label: "2160p (4K)" },
    { value: 1440, label: "1440p" },
    { value: 1080, label: "1080p" },
    { value: 720, label: "720p" },
    { value: 480, label: "480p" },
  ];

  const FRAME_RATE_OPTIONS = [
    { value: null, label: "Original" },
    { value: 60, label: "60 fps" },
    { value: 30, label: "30 fps" },
    { value: 24, label: "24 fps" },
    { value: 15, label: "15 fps" },
  ];

  const isGif = $derived(preferences.videoFormat === "gif");
  const frameRateDescription = $derived(isGif && preferences.frameRate === null ? "GIFs default to 15 fps." : undefined);
</script>

<Field.Group>
  <FormatPicker label="Format" options={formatOptions} bind:value={preferences.videoFormat} />

  {#if !isGif}
    <OptionToggle label="Quality" options={QUALITY_OPTIONS} bind:value={preferences.videoQuality} />
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
