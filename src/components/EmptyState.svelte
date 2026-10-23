<script lang="ts">
  import { FolderOpen, ImagePlus, ShieldCheck } from "@lucide/svelte";

  interface Props {
    imageExtensions: string[];
    videoExtensions: string[];
    addShortcutLabel: string;
    onChooseFiles: () => void;
    onChooseFolder: () => void;
  }

  let { imageExtensions, videoExtensions, addShortcutLabel, onChooseFiles, onChooseFolder }: Props = $props();

  // "jpeg" and "jpg" (and "tif"/"tiff", "heif"/"heic") are the same format to a
  // reader; list each one once.
  const HIDDEN_ALIASES = new Set(["jpeg", "tif", "heif", "mpeg", "m2ts"]);

  function describeExtensions(extensions: string[]): string {
    const visibleExtensions = extensions.filter((extension) => !HIDDEN_ALIASES.has(extension));
    const description = visibleExtensions.map((extension) => extension.toUpperCase()).join(", ");
    return description;
  }
</script>

<section class="empty-state">
  <div class="drop-target">
    <div class="icon-badge" aria-hidden="true">
      <ImagePlus size={28} strokeWidth={1.75} />
    </div>
    <h2 class="heading">Drop images or videos here</h2>
    <p class="subheading">Or choose them yourself. Folders work too.</p>

    <div class="actions">
      <button type="button" class="button button--primary button--large" onclick={onChooseFiles}>
        Choose files…
        <span class="shortcut-hint">{addShortcutLabel}O</span>
      </button>
      <button type="button" class="button button--large" onclick={onChooseFolder}>
        <FolderOpen size={16} aria-hidden="true" />
        Choose a folder…
      </button>
    </div>
  </div>

  <dl class="supported-formats">
    <div>
      <dt>Images</dt>
      <dd>{describeExtensions(imageExtensions)}</dd>
    </div>
    <div>
      <dt>Videos</dt>
      <dd>{describeExtensions(videoExtensions)}</dd>
    </div>
  </dl>

  <p class="privacy-note">
    <ShieldCheck size={14} aria-hidden="true" />
    Everything is converted on this computer. Nothing is uploaded.
  </p>
</section>

<style>
  .empty-state {
    display: grid;
    align-content: center;
    justify-items: center;
    gap: 24px;
    height: 100%;
    padding: 32px;
    overflow-y: auto;
  }

  .drop-target {
    display: grid;
    justify-items: center;
    gap: 8px;
    width: min(560px, 100%);
    padding: 48px 32px;
    border: 2px dashed var(--color-border-strong);
    border-radius: var(--radius-large);
    background: var(--color-surface);
    text-align: center;
  }

  .icon-badge {
    display: grid;
    place-items: center;
    width: 56px;
    height: 56px;
    margin-bottom: 8px;
    border-radius: 16px;
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .heading {
    font-size: var(--font-size-heading);
    font-weight: 600;
  }

  .subheading {
    color: var(--color-text-secondary);
  }

  .actions {
    display: flex;
    flex-wrap: wrap;
    justify-content: center;
    gap: 10px;
    margin-top: 16px;
  }

  .supported-formats {
    display: grid;
    gap: 6px;
    width: min(560px, 100%);
    font-size: var(--font-size-small);
  }

  .supported-formats div {
    display: grid;
    grid-template-columns: 56px 1fr;
    gap: 8px;
  }

  .supported-formats dt {
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .supported-formats dd {
    color: var(--color-text-tertiary);
  }

  .privacy-note {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: var(--font-size-small);
    color: var(--color-text-tertiary);
  }
</style>
