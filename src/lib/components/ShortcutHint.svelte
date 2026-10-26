<script lang="ts">
  import * as Kbd from "$lib/components/ui/kbd/index.js";
  import { shortcutKeys, type Shortcut } from "$lib/shortcuts";

  interface Props {
    shortcut: Shortcut;
    platform: string;
  }

  let { shortcut, platform }: Props = $props();

  const keys = $derived(shortcutKeys(shortcut, platform));
</script>

{#if keys.length > 0}
  <Kbd.Group class="ml-1 opacity-70" aria-hidden="true">
    {#each keys as key (key)}
      <Kbd.Root>{key}</Kbd.Root>
    {/each}
  </Kbd.Group>
{/if}
