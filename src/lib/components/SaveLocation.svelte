<script lang="ts">
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Field from "$lib/components/ui/field/index.js";
  import * as RadioGroup from "$lib/components/ui/radio-group/index.js";
  import type { SaveLocation } from "$lib/preferences.svelte";

  interface Props {
    saveLocation: SaveLocation;
  }

  let { saveLocation = $bindable() }: Props = $props();

  type SaveMode = SaveLocation["mode"];

  const folderPath = $derived(saveLocation.mode === "folder" ? saveLocation.path : null);

  async function chooseFolder(): Promise<void> {
    const chosenPath = await open({ directory: true, title: "Save converted files to" });
    if (typeof chosenPath !== "string") {
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

<Field.Set>
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
      <span class="text-muted-foreground min-w-0 flex-1 truncate text-sm select-text" title={folderPath} dir="rtl">
        {folderPath}
      </span>
      <Button variant="outline" size="sm" onclick={chooseFolder}>
        <FolderOpenIcon />
        Change
      </Button>
    </div>
  {/if}
  <Field.Description>Existing files are never overwritten.</Field.Description>
</Field.Set>
