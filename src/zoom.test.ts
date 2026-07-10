import { describe, expect, it } from "vitest";
import { nextZoom } from "./zoom";

describe("zoom levels", () => {
  it("steps through browser-style levels", () => {
    expect(nextZoom(100, 1)).toBe(110);
    expect(nextZoom(100, -1)).toBe(90);
    expect(nextZoom(111, 1)).toBe(125);
  });

  it("stays within the supported range", () => {
    expect(nextZoom(500, 1)).toBe(500);
    expect(nextZoom(25, -1)).toBe(25);
  });
});
