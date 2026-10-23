<script lang="ts" generics="Value extends string">
  interface SegmentOption {
    value: Value;
    label: string;
  }

  interface Props {
    legend: string;
    name: string;
    options: SegmentOption[];
    value: Value;
  }

  let { legend, name, options, value = $bindable() }: Props = $props();
</script>

<!-- Native radios: arrow keys move between segments and screen readers
     announce it as a group, with no extra code. -->
<fieldset class="field">
  <legend class="field-label">{legend}</legend>
  <div class="segments">
    {#each options as option (option.value)}
      <label class="segment">
        <input type="radio" {name} value={option.value} bind:group={value} />
        <span>{option.label}</span>
      </label>
    {/each}
  </div>
</fieldset>

<style>
  legend {
    margin-bottom: 8px;
  }

  .segments {
    display: flex;
    padding: 2px;
    border-radius: var(--radius-medium);
    background: var(--color-surface-sunken);
  }

  .segment {
    position: relative;
    flex: 1;
    min-width: 0;
  }

  .segment input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .segment span {
    display: block;
    padding: 5px 8px;
    border-radius: var(--radius-small);
    text-align: center;
    font-weight: 500;
    color: var(--color-text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition:
      background var(--duration-quick) var(--easing-standard),
      color var(--duration-quick) var(--easing-standard);
  }

  .segment:hover span {
    color: var(--color-text);
  }

  .segment input:checked + span {
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: var(--shadow-raised);
  }

  .segment input:focus-visible + span {
    box-shadow: var(--focus-ring);
  }

  .segment input:disabled + span {
    opacity: 0.5;
  }
</style>
