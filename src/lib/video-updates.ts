import type { VideoItem } from "./types";

export function replaceVideoInList(videos: VideoItem[], updatedVideo: VideoItem) {
  return videos.map((video) =>
    video.id === updatedVideo.id ? updatedVideo : video,
  );
}
