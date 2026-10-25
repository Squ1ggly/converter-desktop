// Rust commands reject with plain strings; plugins and the browser with Errors.
export function errorMessageOf(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  return message;
}
