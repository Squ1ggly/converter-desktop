<script lang="ts">
  import type { Snippet } from "svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import type { Shortcut } from "$lib/shortcuts";
  import ShortcutHint from "./ShortcutHint.svelte";

  interface Props {
    label: string;
    disabled?: boolean;
    // For toggles, e.g. the toolbar button of the page that's open.
    isPressed?: boolean;
    // Shown in the tooltip with this platform's keys.
    shortcut?: Shortcut;
    platform?: string;
    onclick: () => void;
    children: Snippet;
  }

  let { label, disabled = false, isPressed, shortcut, platform = "", onclick, children }: Props = $props();
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
  <Tooltip.Content>
    {label}
    {#if shortcut}
      <ShortcutHint {shortcut} {platform} />
    {/if}
  </Tooltip.Content>
</Tooltip.Root>
