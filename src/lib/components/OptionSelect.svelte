<script lang="ts">
  import * as Field from "$lib/components/ui/field/index.js";
  import * as Select from "$lib/components/ui/select/index.js";

  interface Props {
    id: string;
    label: string;
    // null means "keep the original".
    options: { value: number | null; label: string }[];
    value: number | null;
    description?: string;
  }

  let { id, label, options, value = $bindable(), description }: Props = $props();

  // Select values are strings; "original" stands in for null.
  const ORIGINAL_VALUE = "original";

  const selectedLabel = $derived(options.find((option) => option.value === value)?.label ?? "");

  function toSelectValue(optionValue: number | null): string {
    const selectValue = optionValue === null ? ORIGINAL_VALUE : String(optionValue);
    return selectValue;
  }

  function selectOption(selectValue: string): void {
    value = selectValue === ORIGINAL_VALUE ? null : Number(selectValue);
  }
</script>

<Field.Field>
  <Field.Label for={id}>{label}</Field.Label>
  <Select.Root type="single" value={toSelectValue(value)} onValueChange={selectOption}>
    <Select.Trigger {id} class="w-full">{selectedLabel}</Select.Trigger>
    <Select.Content>
      {#each options as option (option.label)}
        <Select.Item value={toSelectValue(option.value)} label={option.label} />
      {/each}
    </Select.Content>
  </Select.Root>
  {#if description}
    <Field.Description>{description}</Field.Description>
  {/if}
</Field.Field>
