import type { VideoItem } from "./types";

export type VideoSortOption =
  | "created-desc"
  | "created-asc"
  | "title-asc"
  | "title-desc";

export function sortVideos(videos: VideoItem[], sort: VideoSortOption) {
  return [...videos].sort((left, right) => compareVideos(left, right, sort));
}

function compareVideos(
  left: VideoItem,
  right: VideoItem,
  sort: VideoSortOption,
) {
  switch (sort) {
    case "created-asc":
      return left.createdAt - right.createdAt || compareTitle(left, right);
    case "title-asc":
      return compareTitle(left, right) || right.createdAt - left.createdAt;
    case "title-desc":
      return compareTitle(right, left) || right.createdAt - left.createdAt;
    case "created-desc":
    default:
      return right.createdAt - left.createdAt || compareTitle(left, right);
  }
}

function compareTitle(left: VideoItem, right: VideoItem) {
  return left.title.localeCompare(right.title, undefined, {
    sensitivity: "base",
  });
}
