export function toggleVideoTag(tagIds: string[], tagId: string) {
  return tagIds.includes(tagId)
    ? tagIds.filter((value) => value !== tagId)
    : [...tagIds, tagId];
}

export function removeVideoTag(tagIds: string[], tagId: string) {
  return tagIds.filter((value) => value !== tagId);
}
