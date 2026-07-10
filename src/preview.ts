export const previewKeyMessage = "yamlmdviewer-preview-key";
export const previewZoomMessage = "yamlmdviewer-preview-zoom";

function previewUrl(revision?: number): string {
  const path = revision === undefined ? "empty" : String(revision);
  const colorMode = globalThis.matchMedia?.("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
  return navigator.userAgent.includes("Windows")
    ? `http://yamlmdpreview.localhost/${path}/${colorMode}`
    : `yamlmdpreview://localhost/${path}/${colorMode}`;
}

export function showPreview(frame: HTMLIFrameElement, revision: number): void { frame.src = previewUrl(revision); }
export function clearPreview(frame: HTMLIFrameElement): void { frame.src = previewUrl(); }
