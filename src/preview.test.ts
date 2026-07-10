import { afterEach, describe, expect, it, vi } from "vitest";
import { clearPreview, showPreview } from "./preview";

describe("preview shell", () => {
  afterEach(() => vi.unstubAllGlobals());

  it("loads a revision from the isolated preview protocol", () => {
    const frame = { src: "" } as HTMLIFrameElement;

    showPreview(frame, 42);

    expect(frame.src).toBe("yamlmdpreview://localhost/42/light");
  });

  it("requests dark mode before loading when the system prefers it", () => {
    vi.stubGlobal("matchMedia", vi.fn().mockReturnValue({ matches: true }));
    const frame = { src: "" } as HTMLIFrameElement;

    showPreview(frame, 42);

    expect(frame.src).toBe("yamlmdpreview://localhost/42/dark");
  });

  it("keeps keyboard handling available before a document is open", () => {
    const frame = { src: "" } as HTMLIFrameElement;

    clearPreview(frame);

    expect(frame.src).toBe("yamlmdpreview://localhost/empty/light");
  });
});
