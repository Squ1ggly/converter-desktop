import { marked } from "marked";
import { gfmHeadingId } from "marked-gfm-heading-id";

// The user guide in docs/, bundled at build time so Help works offline and
// always matches the docs on GitHub.
const markdownByPath = import.meta.glob<string>("/docs/*.md", { query: "?raw", import: "default", eager: true });

const PAGE_ORDER = [
  "adding-files",
  "images",
  "videos",
  "converting",
  "saving",
  "shortcuts",
  "settings-and-data",
  "troubleshooting",
  "installing",
];

// Links that leave the user guide (development docs, CONTRIBUTING) open here.
const REPOSITORY_DOCS_URL = "https://github.com/Squ1ggly/converter-desktop/blob/main/docs/";

export interface GuidePage {
  slug: string;
  title: string;
  html: string;
}

export type GuideLink =
  | { kind: "page"; slug: string; anchor: string | null }
  | { kind: "external"; url: string };

// Heading ids match GitHub's, so links like "saving.md#file-names" work in both.
marked.use(gfmHeadingId());

function readTitle(markdown: string, fallbackTitle: string): string {
  const titleLine = markdown.split("\n").find((line) => line.startsWith("# "));
  const title = titleLine?.slice("# ".length).trim() ?? fallbackTitle;
  return title;
}

function loadPage(slug: string): GuidePage | null {
  const markdown = markdownByPath[`/docs/${slug}.md`];
  if (markdown === undefined) {
    return null;
  }

  const page: GuidePage = {
    slug,
    title: readTitle(markdown, slug),
    html: marked.parse(markdown, { async: false }),
  };
  return page;
}

export const GUIDE_PAGES: GuidePage[] = PAGE_ORDER.map(loadPage).filter((page): page is GuidePage => page !== null);

function isGuidePageSlug(slug: string): boolean {
  const isKnown = PAGE_ORDER.includes(slug);
  return isKnown;
}

// Resolves a link inside `currentSlug` the way GitHub would, then decides
// whether it opens another Help page or the browser.
export function resolveGuideLink(href: string, currentSlug: string): GuideLink {
  const resolvedUrl = new URL(href, `${REPOSITORY_DOCS_URL}${currentSlug}.md`);
  const isInsideDocsFolder = resolvedUrl.href.startsWith(REPOSITORY_DOCS_URL);
  const relativePath = resolvedUrl.pathname.slice(new URL(REPOSITORY_DOCS_URL).pathname.length);
  const slug = relativePath.replace(/\.md$/, "");
  const anchor = resolvedUrl.hash ? decodeURIComponent(resolvedUrl.hash.slice(1)) : null;

  if (isInsideDocsFolder && isGuidePageSlug(slug)) {
    return { kind: "page", slug, anchor };
  }

  const externalLink: GuideLink = { kind: "external", url: resolvedUrl.href };
  return externalLink;
}
