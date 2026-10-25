import { toast } from "svelte-sonner";

// Rust commands reject with plain strings; plugins and the browser with Errors.
export function errorMessageOf(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  return message;
}

// Every error toast has a short title saying what failed, with the details under it.
export function showError(title: string, error: unknown): void {
  toast.error(title, { description: errorMessageOf(error) });
}
