<script lang="ts">
  import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
  import ImagePlusIcon from "@lucide/svelte/icons/image-plus";
  import PlusIcon from "@lucide/svelte/icons/plus";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Empty from "$lib/components/ui/empty/index.js";
  import ShortcutHint from "./ShortcutHint.svelte";

  interface Props {
    imageExtensions: string[];
    videoExtensions: string[];
    platform: string;
    onChooseFiles: () => void;
    onChooseFolder: () => void;
  }

  let { imageExtensions, videoExtensions, platform, onChooseFiles, onChooseFolder }: Props = $props();

  // Alternate spellings of a format listed elsewhere (jpeg = jpg, tif = tiff).
  const ALIAS_EXTENSIONS = new Set(["jpeg", "tif", "heif", "mpeg", "m2ts"]);

  function listExtensions(extensions: string[]): string {
    const visibleExtensions = extensions.filter((extension) => !ALIAS_EXTENSIONS.has(extension));
    const extensionList = visibleExtensions.map((extension) => extension.toUpperCase()).join(", ");
    return extensionList;
  }
</script>

<div class="flex h-full items-center justify-center p-8">
  <Empty.Root class="max-w-xl border border-dashed">
    <Empty.Header>
      <Empty.Media variant="icon">
        <ImagePlusIcon />
      </Empty.Media>
      <Empty.Title>Drop images or videos</Empty.Title>
      <Empty.Description>Or add them here. Folders work too.</Empty.Description>
    </Empty.Header>
    <Empty.Content>
      <div class="flex gap-2">
        <Button onclick={onChooseFiles}>
          <PlusIcon />
          Add files
          <ShortcutHint shortcut="open" {platform} />
        </Button>
        <Button variant="outline" onclick={onChooseFolder}>
          <FolderOpenIcon />
          Add folder
        </Button>
      </div>
      <dl class="text-muted-foreground mt-4 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-left text-xs">
        <dt class="text-foreground font-medium">Images</dt>
        <dd>{listExtensions(imageExtensions)}</dd>
        <dt class="text-foreground font-medium">Videos</dt>
        <dd>{listExtensions(videoExtensions)}</dd>
      </dl>
    </Empty.Content>
  </Empty.Root>
</div>
