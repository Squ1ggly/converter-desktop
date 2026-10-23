<script lang="ts">
  import { FolderInput } from "@lucide/svelte";
</script>

<div class="drop-overlay" aria-hidden="true">
  <div class="drop-frame">
    <FolderInput size={32} strokeWidth={1.75} />
    <p>Drop to add</p>
  </div>
</div>

<style>
  .drop-overlay {
    position: fixed;
    inset: 0;
    z-index: 10;
    display: grid;
    padding: 16px;
    background: var(--color-overlay);
    backdrop-filter: blur(4px);
    pointer-events: none;
  }

  .drop-frame {
    display: grid;
    place-content: center;
    justify-items: center;
    gap: 10px;
    border: 2px dashed var(--color-accent);
    border-radius: var(--radius-large);
    color: var(--color-accent);
    font-size: var(--font-size-heading);
    font-weight: 600;
  }
</style>
