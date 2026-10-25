<script lang="ts">
  import { tick } from "svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Button } from "$lib/components/ui/button/index.js";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
  import { GUIDE_PAGES, resolveGuideLink } from "$lib/guide";

  let currentSlug = $state(GUIDE_PAGES[0]?.slug ?? "");
  let article = $state<HTMLElement | null>(null);

  const currentPage = $derived(GUIDE_PAGES.find((page) => page.slug === currentSlug));

  async function showPage(slug: string, anchor: string | null = null): Promise<void> {
    currentSlug = slug;
    await tick();

    const target = anchor ? article?.querySelector(`#${CSS.escape(anchor)}`) : null;
    if (target) {
      target.scrollIntoView();
      return;
    }
    article?.scrollIntoView();
  }

  function handleLinkClick(event: MouseEvent): void {
    const link = event.target instanceof Element ? event.target.closest("a") : null;
    const href = link?.getAttribute("href");
    if (!href) {
      return;
    }

    event.preventDefault();
    const destination = resolveGuideLink(href, currentSlug);
    if (destination.kind === "page") {
      void showPage(destination.slug, destination.anchor);
      return;
    }
    void openUrl(destination.url);
  }
</script>

<div class="grid h-full min-h-0 grid-cols-[13rem_1fr]">
  <nav class="bg-muted/30 flex flex-col gap-0.5 border-r p-3" aria-label="Help pages">
    {#each GUIDE_PAGES as page (page.slug)}
      <Button
        variant="ghost"
        size="sm"
        class="justify-start {page.slug === currentSlug ? 'bg-muted text-foreground' : 'text-muted-foreground'}"
        aria-current={page.slug === currentSlug ? "page" : undefined}
        onclick={() => showPage(page.slug)}
      >
        {page.title}
      </Button>
    {/each}
  </nav>

  <ScrollArea class="min-h-0">
    <!-- Bundled from docs/ at build time, so the HTML is trusted. Clicks are
         handled here so links inside the rendered Markdown navigate in-app. -->
    <!-- Links fire click on Enter too, so no key handler is needed. -->
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <article bind:this={article} onclick={handleLinkClick} class="prose prose-sm prose-neutral dark:prose-invert prose-code:before:content-none prose-code:after:content-none prose-h1:text-2xl prose-h1:font-semibold max-w-2xl scroll-mt-4 px-8 py-6 select-text">
      {@html currentPage?.html ?? ""}
    </article>
  </ScrollArea>
</div>
