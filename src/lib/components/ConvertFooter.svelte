<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { Progress } from "$lib/components/ui/progress/index.js";
  import { Spinner } from "$lib/components/ui/spinner/index.js";
  import { pluralise } from "$lib/formatting";
  import ShortcutHint from "./ShortcutHint.svelte";

  interface Props {
    fileCount: number;
    finishedCount: number;
    isConverting: boolean;
    isCancelling: boolean;
    // Every file has a result; converting again redoes them all.
    isRerun: boolean;
    platform: string;
    onConvert: () => void;
    onCancel: () => void;
  }

  let { fileCount, finishedCount, isConverting, isCancelling, isRerun, platform, onConvert, onCancel }: Props =
    $props();

  const convertLabel = $derived(isRerun ? "Convert again" : `Convert ${pluralise(fileCount, "file")}`);
  const currentFileNumber = $derived(Math.min(finishedCount + 1, fileCount));
</script>

<footer class="bg-background flex flex-col gap-3 border-t p-4">
  {#if isConverting}
    <div class="flex items-center gap-2 text-sm font-medium" aria-live="polite">
      <Spinner />
      Converting {currentFileNumber} of {fileCount}
    </div>
    <Progress value={finishedCount} max={fileCount} aria-label="Overall progress" />
    <Button variant="outline" size="lg" class="w-full" onclick={onCancel} disabled={isCancelling}>
      {isCancelling ? "Cancelling" : "Cancel"}
    </Button>
  {:else}
    <Button size="lg" class="w-full" onclick={onConvert} disabled={fileCount === 0}>
      {convertLabel}
      <ShortcutHint shortcut="convert" {platform} />
    </Button>
  {/if}
</footer>
