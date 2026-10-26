<script lang="ts">
  import * as Field from "$lib/components/ui/field/index.js";
  import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
  import type { FormatOption } from "$lib/backend";
  import { SELECTED_TOGGLE_CLASS, TOGGLE_GROUP_STYLE } from "$lib/styles";

  interface Props {
    label: string;
    options: FormatOption[];
    value: string;
  }

  let { label, options, value = $bindable() }: Props = $props();

  const selectedOption = $derived(options.find((option) => option.id === value));

  // Clicking the selected format again would clear it. A format is always needed.
  function selectFormat(nextValue: string): void {
    if (!nextValue) {
      return;
    }
    value = nextValue;
  }
</script>

<Field.Set>
  <Field.Legend variant="label">{label}</Field.Legend>
  <ToggleGroup.Root
    type="single"
    {...TOGGLE_GROUP_STYLE}
    class="grid w-full grid-cols-4"
    {value}
    onValueChange={selectFormat}
    aria-label={label}
  >
    {#each options as option (option.id)}
      <ToggleGroup.Item
        value={option.id}
        title={option.description}
        class={SELECTED_TOGGLE_CLASS}
      >
        {option.label}
      </ToggleGroup.Item>
    {/each}
  </ToggleGroup.Root>
  <Field.Description>{selectedOption?.description}</Field.Description>
</Field.Set>
