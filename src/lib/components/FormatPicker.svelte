<script lang="ts">
  import * as Field from "$lib/components/ui/field/index.js";
  import * as ToggleGroup from "$lib/components/ui/toggle-group/index.js";
  import type { FormatOption } from "$lib/backend";

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
    variant="outline"
    size="sm"
    spacing={1}
    class="grid w-full grid-cols-4"
    {value}
    onValueChange={selectFormat}
    aria-label={label}
  >
    {#each options as option (option.id)}
      <ToggleGroup.Item
        value={option.id}
        title={option.description}
        class="data-[state=on]:bg-primary data-[state=on]:text-primary-foreground data-[state=on]:border-primary"
      >
        {option.label}
      </ToggleGroup.Item>
    {/each}
  </ToggleGroup.Root>
  <Field.Description class="min-h-10">{selectedOption?.description}</Field.Description>
</Field.Set>
