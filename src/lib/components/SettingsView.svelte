<script lang="ts">
  import { setMode, userPrefersMode } from "mode-watcher";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import type { ConversionCatalog } from "$lib/backend";
  import type { Preferences } from "$lib/preferences.svelte";
  import ImageSettings from "./ImageSettings.svelte";
  import OptionToggle from "./OptionToggle.svelte";
  import SaveLocation from "./SaveLocation.svelte";
  import VideoSettings from "./VideoSettings.svelte";

  interface Props {
    catalog: ConversionCatalog;
    preferences: Preferences;
    isConverting: boolean;
  }

  let { catalog, preferences = $bindable(), isConverting }: Props = $props();

  type ThemeChoice = "system" | "light" | "dark";

  const THEME_OPTIONS: { value: ThemeChoice; label: string }[] = [
    { value: "system", label: "System" },
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
  ];
</script>

<ScrollArea class="h-full">
  <div class="mx-auto flex max-w-xl flex-col gap-8 px-6 py-8">
    <section class="flex flex-col gap-4" aria-labelledby="appearance-heading">
      <h2 id="appearance-heading" class="text-base font-semibold">Appearance</h2>
      <OptionToggle
        label="Theme"
        options={THEME_OPTIONS}
        bind:value={() => userPrefersMode.current, (theme) => setMode(theme)}
      />
    </section>

    <Separator />

    <!-- The same settings as the side panel. Locked while files convert. -->
    <fieldset class="flex min-w-0 flex-col gap-8" disabled={isConverting}>
      <section class="flex flex-col gap-4" aria-labelledby="image-defaults-heading">
        <div>
          <h2 id="image-defaults-heading" class="text-base font-semibold">Images</h2>
          <p class="text-muted-foreground text-sm">Used for every image you convert.</p>
        </div>
        <ImageSettings formatOptions={catalog.imageOutputs} bind:preferences />
      </section>

      <Separator />

      <section class="flex flex-col gap-4" aria-labelledby="video-defaults-heading">
        <div>
          <h2 id="video-defaults-heading" class="text-base font-semibold">Videos</h2>
          <p class="text-muted-foreground text-sm">Used for every video you convert.</p>
        </div>
        <VideoSettings formatOptions={catalog.videoOutputs} bind:preferences />
      </section>

      <Separator />

      <SaveLocation bind:saveLocation={preferences.saveLocation} />
    </fieldset>
  </div>
</ScrollArea>
