import { describe, expect, it } from "vitest";

import { removeVideoTag, toggleVideoTag } from "./video-tags";

describe("video-tags", () => {
  it("adds a missing tag id when toggled on", () => {
    expect(toggleVideoTag(["tag-a"], "tag-b")).toEqual(["tag-a", "tag-b"]);
  });

  it("removes an existing tag id when toggled off", () => {
    expect(toggleVideoTag(["tag-a", "tag-b"], "tag-b")).toEqual(["tag-a"]);
  });

  it("removes only the requested tag id", () => {
    expect(removeVideoTag(["tag-a", "tag-b", "tag-c"], "tag-b")).toEqual([
      "tag-a",
      "tag-c",
    ]);
  });
});
