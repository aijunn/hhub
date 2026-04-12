import { describe, expect, it } from "vitest";

import { resolveLibrarySelection } from "./library-selection";

describe("resolveLibrarySelection", () => {
  it("keeps the first open state empty when nothing is selected", () => {
    expect(resolveLibrarySelection(null, ["a", "b"])).toBeNull();
  });

  it("preserves an existing valid selection", () => {
    expect(resolveLibrarySelection("b", ["a", "b"])).toBe("b");
  });

  it("clears the selection when the previously selected video is gone", () => {
    expect(resolveLibrarySelection("gone", ["a", "b"])).toBeNull();
  });
});
