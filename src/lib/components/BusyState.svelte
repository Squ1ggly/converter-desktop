<script lang="ts">
  import * as Empty from "$lib/components/ui/empty/index.js";
  import { Spinner } from "$lib/components/ui/spinner/index.js";

  interface Props {
    label: string;
  }

  let { label }: Props = $props();
</script>

<div class="flex h-full items-center justify-center p-8">
  <Empty.Root class="max-w-xl" role="status" aria-live="polite">
    <Empty.Header>
      <Empty.Media variant="icon">
        <Spinner role="presentation" aria-label={undefined} aria-hidden="true" />
      </Empty.Media>
      <Empty.Title>{label}</Empty.Title>
    </Empty.Header>
  </Empty.Root>
</div>
