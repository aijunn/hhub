import { describe, expect, it } from "vitest";

import { resolvePostImportSelection } from "./library-view-state";

describe("resolvePostImportSelection", () => {
  it("keeps the current selection when the library already has a selected item", () => {
    expect(
      resolvePostImportSelection({
        firstImportedId: "new-video",
        selectedVideoId: "current-video",
        playbackId: null,
      }),
    ).toBe("current-video");
  });

  it("keeps the current playback target selection when a video is already loaded", () => {
    expect(
      resolvePostImportSelection({
        firstImportedId: "new-video",
        selectedVideoId: null,
        playbackId: "playing-video",
      }),
    ).toBe("playing-video");
  });

  it("selects the first imported video when nothing is currently selected or loaded", () => {
    expect(
      resolvePostImportSelection({
        firstImportedId: "new-video",
        selectedVideoId: null,
        playbackId: null,
      }),
    ).toBe("new-video");
  });

  it("returns null when there is no imported video to select", () => {
    expect(
      resolvePostImportSelection({
        firstImportedId: null,
        selectedVideoId: null,
        playbackId: null,
      }),
    ).toBeNull();
  });
});
