import { describe, expect, it } from "vitest";

import { pickRandomVideo } from "./random-video";

describe("pickRandomVideo", () => {
  it("returns null when the list is empty", () => {
    expect(pickRandomVideo([], null, () => 0)).toBeNull();
  });

  it("returns the only video even when it is current", () => {
    const onlyVideo = { id: "only" };

    expect(pickRandomVideo([onlyVideo], "only", () => 0)).toBe(onlyVideo);
  });

  it("avoids the current video when alternatives exist", () => {
    const videos = [{ id: "current" }, { id: "next" }, { id: "other" }];
    const selected = pickRandomVideo(videos, "current", () => 0);

    expect(selected).not.toBeNull();
    expect(selected?.id).toBe("next");
  });

  it("uses the injected random function to select a candidate", () => {
    const videos = [{ id: "current" }, { id: "middle" }, { id: "last" }];
    const selected = pickRandomVideo(videos, "current", () => 0.99);

    expect(selected).not.toBeNull();
    expect(selected?.id).toBe("last");
  });
});
