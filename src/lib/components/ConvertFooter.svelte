<script lang="ts">
  import { Button } from "$lib/components/ui/button/index.js";
  import { Progress } from "$lib/components/ui/progress/index.js";
  import { Spinner } from "$lib/components/ui/spinner/index.js";
  import { hasLostFocus } from "$lib/focus";
  import { formatCount, pluralise } from "$lib/formatting";
  import ShortcutHint from "./ShortcutHint.svelte";

  interface Props {
    fileCount: number;
    finishedCount: number;
    isConverting: boolean;
    // Files are still being added; converting waits for them.
    isAdding: boolean;
    isCancelling: boolean;
    // Every file has a result; converting again redoes them all.
    isRerun: boolean;
    platform: string;
    onConvert: () => void;
    onCancel: () => void;
  }

  let { fileCount, finishedCount, isConverting, isAdding, isCancelling, isRerun, platform, onConvert, onCancel }: Props =
    $props();

  const convertLabel = $derived(isRerun ? "Convert again" : `Convert ${pluralise(fileCount, "file")}`);
  const currentFileNumber = $derived(Math.min(finishedCount + 1, fileCount));

  let convertButton = $state<HTMLButtonElement | null>(null);
  let cancelButton = $state<HTMLButtonElement | null>(null);
  let wasConverting: boolean | null = null;

  // Convert and Cancel replace each other, which drops focus. Hand it to the
  // button now shown, unless the user has moved on to something else.
  $effect(() => {
    const isNowConverting = isConverting;
    const hasChanged = wasConverting !== null && wasConverting !== isNowConverting;
    wasConverting = isNowConverting;
    if (!hasChanged || !hasLostFocus()) {
      return;
    }

    const replacementButton = isNowConverting ? cancelButton : convertButton;
    replacementButton?.focus();
  });
</script>

<footer class="bg-background flex flex-col gap-3 border-t p-4">
  {#if isConverting}
    <div class="flex items-center gap-2 text-sm font-medium" aria-live="polite">
      <Spinner />
      Converting {formatCount(currentFileNumber)} of {formatCount(fileCount)}
    </div>
    <Progress value={finishedCount} max={fileCount} aria-label="Overall progress" />
    <Button bind:ref={cancelButton} variant="outline" size="lg" class="w-full" onclick={onCancel} disabled={isCancelling}>
      {isCancelling ? "Cancelling" : "Cancel"}
    </Button>
  {:else}
    <Button bind:ref={convertButton} size="lg" class="w-full" onclick={onConvert} disabled={fileCount === 0 || isAdding}>
      {convertLabel}
      <ShortcutHint shortcut="convert" {platform} />
    </Button>
  {/if}
</footer>
