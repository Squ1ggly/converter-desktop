// True when the focused element was removed or disabled and focus fell back to the page.
export function hasLostFocus(): boolean {
  const focusedElement = document.activeElement;
  const isLost = focusedElement === null || focusedElement === document.body;
  return isLost;
}
