import { describe, expect, it } from "vitest";

import type { VideoItem } from "./types";
import { sortVideos } from "./video-sort";

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

describe("sortVideos", () => {
  const videos = [
    createVideo({ id: "b", title: "Bravo", createdAt: 2 }),
    createVideo({ id: "a", title: "Alpha", createdAt: 1 }),
    createVideo({ id: "c", title: "Charlie", createdAt: 3 }),
  ];

  it("sorts by newest added first", () => {
    expect(sortVideos(videos, "created-desc").map((video) => video.id)).toEqual([
      "c",
      "b",
      "a",
    ]);
  });

  it("sorts by oldest added first", () => {
    expect(sortVideos(videos, "created-asc").map((video) => video.id)).toEqual([
      "a",
      "b",
      "c",
    ]);
  });

  it("sorts by title ascending", () => {
    expect(sortVideos(videos, "title-asc").map((video) => video.id)).toEqual([
      "a",
      "b",
      "c",
    ]);
  });

  it("sorts by play count descending", () => {
    const videosByPlays = [
      createVideo({ id: "low", title: "Low", createdAt: 3, playCount: 1 }),
      createVideo({ id: "high", title: "High", createdAt: 1, playCount: 8 }),
      createVideo({ id: "tie-newer", title: "Tie Newer", createdAt: 5, playCount: 4 }),
      createVideo({ id: "tie-older", title: "Tie Older", createdAt: 2, playCount: 4 }),
    ];

    expect(
      sortVideos(videosByPlays, "play-count-desc").map((video) => video.id),
    ).toEqual(["high", "tie-newer", "tie-older", "low"]);
  });

  it("sorts by play count ascending", () => {
    const videosByPlays = [
      createVideo({ id: "middle", title: "Middle", createdAt: 3, playCount: 4 }),
      createVideo({ id: "none-newer", title: "None Newer", createdAt: 5, playCount: 0 }),
      createVideo({ id: "none-older", title: "None Older", createdAt: 2, playCount: 0 }),
      createVideo({ id: "high", title: "High", createdAt: 1, playCount: 8 }),
    ];

    expect(
      sortVideos(videosByPlays, "play-count-asc").map((video) => video.id),
    ).toEqual(["none-newer", "none-older", "middle", "high"]);
  });
});
