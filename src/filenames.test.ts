import { describe, expect, it } from "vitest";
import { defaultHtmlPath } from "./filenames";

describe("defaultHtmlPath", () => {
  it("replaces the full YAML Markdown suffix case-insensitively", () => {
    expect(defaultHtmlPath("C:\\docs\\report.YAML.MD")).toBe("C:\\docs\\report.html");
  });
});
