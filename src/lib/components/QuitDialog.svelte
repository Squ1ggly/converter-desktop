<script lang="ts">
  import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";

  interface Props {
    open: boolean;
    onQuit: () => void;
  }

  let { open = $bindable(), onQuit }: Props = $props();
</script>

<AlertDialog.Root bind:open>
  <AlertDialog.Content>
    <AlertDialog.Header>
      <AlertDialog.Title>Quit while converting?</AlertDialog.Title>
      <AlertDialog.Description>
        Files still converting are cancelled. Their unfinished output is removed.
      </AlertDialog.Description>
    </AlertDialog.Header>
    <AlertDialog.Footer>
      <AlertDialog.Cancel>Keep converting</AlertDialog.Cancel>
      <AlertDialog.Action variant="destructive" onclick={onQuit}>Quit</AlertDialog.Action>
    </AlertDialog.Footer>
  </AlertDialog.Content>
</AlertDialog.Root>
