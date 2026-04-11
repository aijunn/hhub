import { describe, expect, it } from "vitest";

import { getTagTheme } from "./tag-theme";

describe("getTagTheme", () => {
  it("returns a stable theme for the same tag label", () => {
    expect(getTagTheme("旅行")).toEqual(getTagTheme("旅行"));
  });

  it("returns the full theme token set", () => {
    expect(getTagTheme("收藏")).toMatchObject({
      background: expect.any(String),
      border: expect.any(String),
      text: expect.any(String),
      accent: expect.any(String),
    });
  });

  it("spreads nearby labels across different palette entries", () => {
    expect(getTagTheme("工作")).not.toEqual(getTagTheme("学习"));
  });
});
