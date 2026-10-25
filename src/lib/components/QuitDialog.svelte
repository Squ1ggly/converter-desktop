<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
  import { Spinner } from "$lib/components/ui/spinner/index.js";

  interface Props {
    open: boolean;
    // Quit was chosen: running files are being cancelled before the window closes.
    isQuitting: boolean;
    onQuit: () => void;
  }

  let { open = $bindable(), isQuitting, onQuit }: Props = $props();

  function keepOpenWhileQuitting(event: KeyboardEvent): void {
    if (isQuitting) {
      event.preventDefault();
    }
  }
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content onEscapeKeydown={keepOpenWhileQuitting}>
    <AlertDialog.Header>
      <AlertDialog.Title>Quit while converting?</AlertDialog.Title>
      <AlertDialog.Description>
        Files still converting are cancelled. Their unfinished output is removed.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel disabled={isQuitting}>Keep converting</AlertDialog.Cancel>
      <AlertDialog.Action variant="destructive" disabled={isQuitting} onclick={onQuit}>
        {#if isQuitting}
          <Spinner role="presentation" aria-label={undefined} aria-hidden="true" />
          Quitting
        {:else}
          Quit
        {/if}
      </AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
