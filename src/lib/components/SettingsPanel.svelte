<script lang="ts">
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import type { ConversionCatalog } from "$lib/backend";
  import type { Preferences } from "$lib/preferences";
  import ConvertFooter from "./ConvertFooter.svelte";
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
    platform: string;
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
    platform,
    onConvert,
    onCancel,
  }: Props = $props();
</script>

<aside class="bg-muted/30 flex min-h-0 flex-col border-l" aria-label="Settings">
  <ScrollArea class="min-h-0 flex-1">
    <!-- A disabled fieldset locks every control while files convert. -->
    <fieldset class="flex min-w-0 flex-col gap-6 p-5" disabled={isConverting}>
      {#if hasImages}
        <section class="flex flex-col gap-4" aria-labelledby="image-settings-heading">
          <h2 id="image-settings-heading" class="text-sm font-semibold">Images</h2>
          <ImageSettings formatOptions={catalog.imageOutputs} bind:preferences />
        </section>
        <Separator />
      {/if}
      {#if hasVideos}
        <section class="flex flex-col gap-4" aria-labelledby="video-settings-heading">
          <h2 id="video-settings-heading" class="text-sm font-semibold">Videos</h2>
          <VideoSettings formatOptions={catalog.videoOutputs} bind:preferences />
        </section>
        <Separator />
      {/if}
      <SaveLocation bind:saveLocation={preferences.saveLocation} />
    </fieldset>
  </ScrollArea>

  <ConvertFooter {fileCount} {finishedCount} {isConverting} {isCancelling} {isRerun} {platform} {onConvert} {onCancel} />
</aside>
