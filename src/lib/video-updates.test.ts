import { describe, expect, it } from "vitest";

import type { VideoItem } from "./types";
import { replaceVideoInList } from "./video-updates";

function createVideo(overrides: Partial<VideoItem>): VideoItem {
  return {
    id: "video-1",
    title: "Alpha",
    fileName: "alpha.mp4",
    storedPath: "/tmp/alpha.mp4",
    mimeType: "video/mp4",
    fileSize: 1,
    durationMs: null,
    isFavorite: false,
    tagIds: [],
    createdAt: 1,
    updatedAt: 1,
    playCount: 0,
    ...overrides,
  };
}

describe("replaceVideoInList", () => {
  it("replaces the matching video without changing list order", () => {
    const videos = [
      createVideo({ id: "first", title: "First" }),
      createVideo({ id: "second", title: "Second" }),
    ];

    const updated = createVideo({ id: "first", title: "First renamed" });
    const nextVideos = replaceVideoInList(videos, updated);

    expect(nextVideos.map((video) => video.id)).toEqual(["first", "second"]);
    expect(nextVideos[0].title).toBe("First renamed");
  });
});
