<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  interface Props {
    label: string;
    disabled?: boolean;
    onclick: () => void;
    children: Snippet;
  }

  let { label, disabled = false, onclick, children }: Props = $props();
</script>

<Tooltip.Root>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      <Button {...props} variant="ghost" size="icon-sm" aria-label={label} {disabled} {onclick}>
        {@render children()}
      </Button>
    {/snippet}
  </Tooltip.Trigger>
  <Tooltip.Content>{label}</Tooltip.Content>
</Tooltip.Root>
