<script lang="ts">
  import type { FormatOption } from "../lib/backend";

  interface Props {
    legend: string;
    name: string;
    options: FormatOption[];
    value: string;
  }

  let { legend, name, options, value = $bindable() }: Props = $props();

  const selectedOption = $derived(options.find((option) => option.id === value));
  const descriptionId = $derived(`${name}-description`);
</script>

<fieldset class="field" aria-describedby={descriptionId}>
  <legend class="field-label">{legend}</legend>
  <div class="format-grid">
    {#each options as option (option.id)}
      <label class="format-chip" title={option.description}>
        <input type="radio" {name} value={option.id} bind:group={value} />
        <span>{option.label}</span>
      </label>
    {/each}
  </div>
  <p id={descriptionId} class="field-help" aria-live="polite">{selectedOption?.description ?? ""}</p>
</fieldset>

<style>
  legend {
    margin-bottom: 8px;
  }

  .format-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(64px, 1fr));
    gap: 6px;
  }

  .format-chip {
    position: relative;
  }

  .format-chip input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .format-chip span {
    display: block;
    padding: 6px 4px;
    border: 1px solid var(--color-border-strong);
    border-radius: var(--radius-small);
    background: var(--color-surface);
    text-align: center;
    font-weight: 600;
    font-size: var(--font-size-small);
    letter-spacing: 0.02em;
    transition:
      background var(--duration-quick) var(--easing-standard),
      border-color var(--duration-quick) var(--easing-standard),
      color var(--duration-quick) var(--easing-standard);
  }

  .format-chip:hover span {
    background: var(--color-surface-hover);
  }

  .format-chip input:checked + span {
    border-color: var(--color-accent);
    background: var(--color-accent-subtle);
    color: var(--color-accent);
  }

  .format-chip input:focus-visible + span {
    box-shadow: var(--focus-ring);
  }

  .format-chip input:disabled + span {
    opacity: 0.5;
  }

  /* Keeps the panel from jumping as descriptions of different lengths swap. */
  .field-help {
    min-height: calc(2 * 1.45em);
  }
</style>
