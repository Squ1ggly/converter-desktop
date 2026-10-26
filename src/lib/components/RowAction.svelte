<script lang="ts">
  import type { Snippet } from "svelte";
  import type { Tooltip as TooltipPrimitive } from "bits-ui";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  interface Props {
    label: string;
    // One tooltip shared by every row, so a long list doesn't create one per button.
    tooltip: TooltipPrimitive.Tether<string>;
    onclick: () => void;
    children: Snippet;
  }

  let { label, tooltip, onclick, children }: Props = $props();
</script>

<Tooltip.Trigger tether={tooltip} payload={label}>
  {#snippet child({ props })}
    <Button {...props} variant="ghost" size="icon-sm" aria-label={label} {onclick}>
      {@render children()}
    </Button>
  {/snippet}
</Tooltip.Trigger>
