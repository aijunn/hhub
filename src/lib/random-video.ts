export function pickRandomVideo<T extends { id: string }>(
  videos: T[],
  currentId: string | null,
  random: () => number = Math.random,
) {
  if (videos.length === 0) {
    return null;
  }

  const candidates =
    currentId && videos.length > 1
      ? videos.filter((video) => video.id !== currentId)
      : videos;
  const index = Math.min(
    Math.floor(random() * candidates.length),
    candidates.length - 1,
  );

  return candidates[index] ?? null;
}
