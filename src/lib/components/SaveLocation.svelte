<script lang="ts">
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Field from "$lib/components/ui/field/index.js";
  import * as RadioGroup from "$lib/components/ui/radio-group/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import { showError } from "$lib/errors";
  import type { SaveLocation } from "$lib/preferences";

  interface Props {
    saveLocation: SaveLocation;
  }

  let { saveLocation = $bindable() }: Props = $props();

  type SaveMode = SaveLocation["mode"];

  const folderPath = $derived(saveLocation.mode === "folder" ? saveLocation.path : null);

  async function chooseFolder(): Promise<void> {
    let chosenPath: string | null = null;
    try {
      chosenPath = await open({ directory: true, title: "Save converted files to" });
    } catch (error) {
      showError("Couldn't open the folder picker", error);
    }
    if (chosenPath === null) {
      return;
    }
    saveLocation = { mode: "folder", path: chosenPath };
  }

  // Picking "Folder" without one chosen opens the picker. Cancelling keeps the old choice.
  async function selectMode(mode: string): Promise<void> {
    if (mode === "besideOriginal") {
      saveLocation = { mode };
      return;
    }
    if (!folderPath) {
      await chooseFolder();
    }
  }

  const selectedMode: SaveMode = $derived(saveLocation.mode);
</script>

<!-- A fieldset is as wide as its content by default, which would let a long path push Change out of view. -->
<Field.Set class="min-w-0">
  <Field.Legend variant="label">Save to</Field.Legend>
  <!-- Bound through the saved choice, so the radio only moves once a folder is picked. -->
  <RadioGroup.Root bind:value={() => selectedMode, (mode) => void selectMode(mode)}>
    <Field.Field orientation="horizontal">
      <RadioGroup.Item value="besideOriginal" id="save-beside-original" />
      <Field.Label for="save-beside-original" class="font-normal">Same folder as original</Field.Label>
    </Field.Field>
    <Field.Field orientation="horizontal">
      <RadioGroup.Item value="folder" id="save-to-folder" />
      <Field.Label for="save-to-folder" class="font-normal">Another folder</Field.Label>
    </Field.Field>
  </RadioGroup.Root>

  {#if folderPath}
    <div class="flex min-w-0 items-center gap-2 pl-6">
      <!-- Right-to-left only moves the ellipsis to the start, so the folder's own
           name stays in view. The path itself is isolated so it still reads left to right. -->
      <Tooltip.Root>
        <Tooltip.Trigger>
          {#snippet child({ props })}
            <span {...props} class="text-muted-foreground min-w-0 flex-1 truncate text-left text-sm select-text" dir="rtl">
              <bdi dir="ltr">{folderPath}</bdi>
            </span>
          {/snippet}
        </Tooltip.Trigger>
        <Tooltip.Content class="max-w-sm break-all">{folderPath}</Tooltip.Content>
      </Tooltip.Root>
      <Button variant="outline" size="sm" onclick={chooseFolder}>
        <FolderOpenIcon />
        Change
      </Button>
    </div>
  {/if}
  <Field.Description>Existing files are never overwritten.</Field.Description>
</Field.Set>
