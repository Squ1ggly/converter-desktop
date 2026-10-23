<script lang="ts">
  import type { FormatOption, VideoQuality } from "../lib/backend";
  import type { Preferences } from "../lib/preferences.svelte";
  import FormatPicker from "./FormatPicker.svelte";
  import SegmentedControl from "./SegmentedControl.svelte";

  interface Props {
    formatOptions: FormatOption[];
    preferences: Preferences;
  }

  let { formatOptions, preferences = $bindable() }: Props = $props();

  const QUALITY_OPTIONS: { value: VideoQuality; label: string }[] = [
    { value: "high", label: "High" },
    { value: "balanced", label: "Balanced" },
    { value: "small", label: "Smaller file" },
  ];

  const RESOLUTION_OPTIONS: { value: number | null; label: string }[] = [
    { value: null, label: "Same as original" },
    { value: 2160, label: "4K (2160p)" },
    { value: 1440, label: "1440p" },
    { value: 1080, label: "1080p (Full HD)" },
    { value: 720, label: "720p (HD)" },
    { value: 480, label: "480p" },
  ];

  const FRAME_RATE_OPTIONS: { value: number | null; label: string }[] = [
    { value: null, label: "Same as original" },
    { value: 60, label: "60 fps" },
    { value: 30, label: "30 fps" },
    { value: 24, label: "24 fps" },
    { value: 15, label: "15 fps" },
  ];

  const isGif = $derived(preferences.videoFormat === "gif");
</script>

<section class="settings-section" aria-labelledby="video-settings-title">
  <h2 id="video-settings-title" class="section-title">Videos</h2>

  <FormatPicker legend="Convert to" name="video-format" options={formatOptions} bind:value={preferences.videoFormat} />

  {#if !isGif}
    <SegmentedControl legend="Quality" name="video-quality" options={QUALITY_OPTIONS} bind:value={preferences.videoQuality} />
  {/if}

  <div class="field">
    <label class="field-label" for="video-resolution">Resolution</label>
    <select id="video-resolution" class="select-input" bind:value={preferences.maxHeight}>
      {#each RESOLUTION_OPTIONS as option (option.label)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
    <p class="field-help">Only ever makes videos smaller, never larger.</p>
  </div>

  <div class="field">
    <label class="field-label" for="video-frame-rate">Frame rate</label>
    <select id="video-frame-rate" class="select-input" bind:value={preferences.frameRate}>
      {#each FRAME_RATE_OPTIONS as option (option.label)}
        <option value={option.value}>{option.label}</option>
      {/each}
    </select>
    {#if isGif && preferences.frameRate === null}
      <p class="field-help">GIFs use 15 fps unless you pick a rate, to keep files small.</p>
    {/if}
  </div>
</section>
