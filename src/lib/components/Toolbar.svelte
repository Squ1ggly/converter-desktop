<script lang="ts">
  import { Plus } from "@lucide/svelte";

  interface Props {
    isMac: boolean;
    hasFiles: boolean;
    isConverting: boolean;
    addShortcutLabel: string;
    onAddFiles: () => void;
    onClear: () => void;
  }

  let { isMac, hasFiles, isConverting, addShortcutLabel, onAddFiles, onClear }: Props = $props();
</script>

<!-- data-tauri-drag-region lets the window be moved by dragging the toolbar,
     since on macOS it replaces the native title bar. -->
<header class="toolbar" class:toolbar--mac={isMac} data-tauri-drag-region>
  <h1 class="app-title" data-tauri-drag-region>Converter</h1>

  {#if hasFiles}
    <div class="actions">
      <button type="button" class="button button--quiet" onclick={onClear} disabled={isConverting}>Clear list</button>
      <button type="button" class="button" onclick={onAddFiles} disabled={isConverting} title="Add files ({addShortcutLabel}O)">
        <Plus size={14} aria-hidden="true" />
        Add files
      </button>
    </div>
  {/if}
</header>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--toolbar-height);
    padding: 0 16px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }

  /* Leaves room for the macOS close/minimise/zoom buttons. */
  .toolbar--mac {
    padding-left: 84px;
  }

  .app-title {
    font-size: var(--font-size-title);
    font-weight: 600;
  }

  .actions {
    display: flex;
    gap: 8px;
  }
</style>
