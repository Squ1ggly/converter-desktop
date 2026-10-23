<script lang="ts">
  import { FolderOpen } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import type { SaveLocation } from "../lib/preferences.svelte";

  interface Props {
    saveLocation: SaveLocation;
  }

  let { saveLocation = $bindable() }: Props = $props();

  const folderPath = $derived(saveLocation.mode === "folder" ? saveLocation.path : null);

  async function chooseFolder(): Promise<void> {
    const chosenPath = await open({ directory: true, title: "Save converted files in…" });
    if (typeof chosenPath !== "string") {
      return;
    }

    saveLocation = { mode: "folder", path: chosenPath };
  }

  // Picking "In a folder" with no folder yet opens the picker straight away;
  // dismissing it leaves the previous choice in place.
  async function selectFolderMode(): Promise<void> {
    if (folderPath) {
      return;
    }
    await chooseFolder();
  }
</script>

<fieldset class="field">
  <legend class="field-label">Save to</legend>

  <label class="choice">
    <input
      type="radio"
      name="save-location"
      checked={saveLocation.mode === "besideOriginal"}
      onchange={() => (saveLocation = { mode: "besideOriginal" })}
    />
    <span>Same folder as each original</span>
  </label>

  <label class="choice">
    <input type="radio" name="save-location" checked={saveLocation.mode === "folder"} onchange={selectFolderMode} />
    <span>A folder I choose</span>
  </label>

  {#if folderPath}
    <div class="folder-row">
      <span class="folder-path selectable" title={folderPath}>{folderPath}</span>
      <button type="button" class="button" onclick={chooseFolder}>
        <FolderOpen size={14} aria-hidden="true" />
        Change…
      </button>
    </div>
  {/if}

  <p class="field-help">Existing files are never overwritten. Copies get a number, like “photo (2).jpg”.</p>
</fieldset>

<style>
  legend {
    margin-bottom: 4px;
  }

  .choice {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .choice input {
    accent-color: var(--color-accent);
    margin: 0;
  }

  .folder-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-left: 22px;
    min-width: 0;
  }

  .folder-path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    direction: rtl;
    text-align: left;
    color: var(--color-text-secondary);
  }
</style>
