export function resolveLibrarySelection(
  currentSelectionId: string | null,
  availableVideoIds: string[],
) {
  if (!currentSelectionId) {
    return null;
  }

  return availableVideoIds.includes(currentSelectionId) ? currentSelectionId : null;
}
