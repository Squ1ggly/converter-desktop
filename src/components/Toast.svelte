<script lang="ts">
  import { Info, X } from "@lucide/svelte";

  interface Props {
    message: string;
    onDismiss: () => void;
  }

  let { message, onDismiss }: Props = $props();
</script>

<div class="toast" role="status">
  <Info size={16} aria-hidden="true" />
  <p class="selectable">{message}</p>
  <button type="button" class="icon-button" onclick={onDismiss} aria-label="Dismiss">
    <X size={14} />
  </button>
</div>

<style>
  .toast {
    position: fixed;
    left: 50%;
    bottom: 24px;
    z-index: 20;
    display: flex;
    align-items: center;
    gap: 10px;
    max-width: min(560px, calc(100vw - 48px));
    padding: 8px 8px 8px 14px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-medium);
    background: var(--color-surface);
    box-shadow: 0 8px 24px rgb(0 0 0 / 0.16);
    transform: translateX(-50%);
    animation: rise 180ms var(--easing-standard);
  }

  p {
    flex: 1;
  }

  @keyframes rise {
    from {
      opacity: 0;
      transform: translate(-50%, 8px);
    }
  }
</style>
