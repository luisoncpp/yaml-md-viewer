import { describe, expect, it } from "vitest";
import { clearPreview, previewKeyMessage, previewZoomMessage, showPreview } from "./preview";

describe("preview shell", () => {
  it("injects the keyboard bridge without changing the exported HTML", () => {
    const frame = { srcdoc: "" } as HTMLIFrameElement;
    const compiled = "<!doctype html><html><head><title>Example</title></head><body>Body</body></html>";

    showPreview(frame, compiled);

    expect(frame.srcdoc).toContain(previewKeyMessage);
    expect(frame.srcdoc).toContain('event.key==="F11"');
    expect(frame.srcdoc).toContain('event.key==="Escape"');
    expect(frame.srcdoc).toContain(previewZoomMessage);
    expect(frame.srcdoc).toContain('event.ctrlKey');
    expect(frame.srcdoc).toContain('addEventListener("wheel"');
    expect(compiled).not.toContain(previewKeyMessage);
  });

  it("keeps keyboard handling available before a document is open", () => {
    const frame = { srcdoc: "" } as HTMLIFrameElement;

    clearPreview(frame);

    expect(frame.srcdoc).toContain(previewKeyMessage);
  });
});
