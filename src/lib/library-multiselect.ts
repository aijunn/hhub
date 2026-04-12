export function toggleMultiSelectId(selectedIds: string[], id: string) {
  return selectedIds.includes(id)
    ? selectedIds.filter((value) => value !== id)
    : [...selectedIds, id];
}

export function toggleSelectAll(selectedIds: string[], allIds: string[]) {
  const allSelected =
    allIds.length > 0 && allIds.every((id) => selectedIds.includes(id));

  return allSelected ? [] : [...allIds];
}

export function deriveBatchPaneMode(selectedIds: string[]) {
  if (selectedIds.length === 0) {
    return "empty";
  }

  if (selectedIds.length === 1) {
    return "single";
  }

  return "batch";
}
