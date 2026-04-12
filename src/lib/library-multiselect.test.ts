import { describe, expect, it } from "vitest";

import {
  deriveBatchPaneMode,
  toggleMultiSelectId,
  toggleSelectAll,
} from "./library-multiselect";

describe("library multiselect helpers", () => {
  it("toggles a row id into and out of selection", () => {
    expect(toggleMultiSelectId([], "a")).toEqual(["a"]);
    expect(toggleMultiSelectId(["a"], "a")).toEqual([]);
  });

  it("selects all when not all are selected and clears when all are selected", () => {
    expect(toggleSelectAll(["a"], ["a", "b"])).toEqual(["a", "b"]);
    expect(toggleSelectAll(["a", "b"], ["a", "b"])).toEqual([]);
  });

  it("derives the right pane mode from selection count", () => {
    expect(deriveBatchPaneMode([])).toBe("empty");
    expect(deriveBatchPaneMode(["a"])).toBe("single");
    expect(deriveBatchPaneMode(["a", "b"])).toBe("batch");
  });
});
