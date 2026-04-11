type ResolvePostImportSelectionOptions = {
  firstImportedId: string | null;
  selectedVideoId: string | null;
  playbackId: string | null;
};

export function resolvePostImportSelection({
  firstImportedId,
  selectedVideoId,
  playbackId,
}: ResolvePostImportSelectionOptions) {
  if (selectedVideoId) {
    return selectedVideoId;
  }

  if (playbackId) {
    return playbackId;
  }

  return firstImportedId;
}
