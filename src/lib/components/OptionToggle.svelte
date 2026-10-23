<script lang="ts" generics="Value extends string">
  import * as Field from "$lib/components/ui/field/index.js";
  import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";

  interface Props {
    label: string;
    options: { value: Value; label: string }[];
    value: Value;
  }

  let { label, options, value = $bindable() }: Props = $props();

  // A toggle group lets you deselect the active item; one option must stay chosen.
  function selectOption(nextValue: string): void {
    const matchingOption = options.find((option) => option.value === nextValue);
    if (!matchingOption) {
      return;
    }
    value = matchingOption.value;
  }
</script>

<Field.Set>
  <Field.Legend variant="label">{label}</Field.Legend>
  <ToggleGroup.Root type="single" variant="outline" class="w-full" {value} onValueChange={selectOption} aria-label={label}>
    {#each options as option (option.value)}
      <ToggleGroup.Item value={option.value} class="flex-1">{option.label}</ToggleGroup.Item>
    {/each}
  </ToggleGroup.Root>
</Field.Set>
