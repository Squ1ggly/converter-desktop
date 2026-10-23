<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";

  interface Props {
    label: string;
    disabled?: boolean;
    // For toggles, e.g. the toolbar button of the page that's open.
    isPressed?: boolean;
    onclick: () => void;
    children: Snippet;
  }

  let { label, disabled = false, isPressed, onclick, children }: Props = $props();
</script>

<Tooltip.Root>
  <Tooltip.Trigger>
    {#snippet child({ props })}
      <Button
        {...props}
        variant="ghost"
        size="icon-sm"
        class={isPressed ? "bg-muted text-foreground" : ""}
        aria-label={label}
        aria-pressed={isPressed}
        {disabled}
        {onclick}
      >
        {@render children()}
      </Button>
    {/snippet}
  </Tooltip.Trigger>
  <Tooltip.Content>{label}</Tooltip.Content>
</Tooltip.Root>
