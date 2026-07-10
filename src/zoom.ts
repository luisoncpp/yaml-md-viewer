import {
  onMenuZoomIn,
  onMenuZoomOut,
  onMenuZoomReset,
  setZoom,
} from "./api";
import { previewApplyZoomMessage, previewZoomMessage } from "./preview";

export const zoomLevels = [25, 33, 50, 67, 75, 80, 90, 100, 110, 125, 150, 175, 200, 250, 300, 400, 500] as const;

export function nextZoom(current: number, direction: -1 | 1): number {
  if (direction > 0) return zoomLevels.find(level => level > current) ?? zoomLevels.at(-1)!;
  return [...zoomLevels].reverse().find(level => level < current) ?? zoomLevels[0];
}

export function setupZoom(preview: HTMLIFrameElement): void {
  let percentage = 100;
  let pending = Promise.resolve();

  const applyToPreview = () => {
    preview.contentWindow?.postMessage({
      type: previewApplyZoomMessage,
      percentage,
    }, "*");
  };
  const apply = (next: number) => {
    percentage = next;
    applyToPreview();
    pending = pending.then(() => setZoom(next)).then(() => undefined).catch(() => undefined);
  };
  const step = (direction: -1 | 1) => apply(nextZoom(percentage, direction));
  const handleKey = (key: string, ctrlKey: boolean, altKey: boolean, preventDefault: () => void) => {
    if (!ctrlKey || altKey || !["+", "=", "-", "0"].includes(key)) return;
    preventDefault();
    if (key === "-") step(-1);
    else if (key === "0") apply(100);
    else step(1);
  };

  window.addEventListener("keydown", event => {
    handleKey(event.key, event.ctrlKey, event.altKey, () => event.preventDefault());
  });
  window.addEventListener("wheel", event => {
    if (!event.ctrlKey || event.deltaY === 0) return;
    event.preventDefault();
    step(event.deltaY < 0 ? 1 : -1);
  }, { passive: false });
  window.addEventListener("message", event => {
    if (event.source !== preview.contentWindow || event.data?.type !== previewZoomMessage) return;
    if (typeof event.data.deltaY === "number" && event.data.deltaY !== 0) {
      step(event.data.deltaY < 0 ? 1 : -1);
    } else if (typeof event.data.key === "string") {
      handleKey(event.data.key, true, false, () => undefined);
    }
  });
  preview.addEventListener("load", applyToPreview);

  void Promise.all([
    onMenuZoomIn(() => step(1)),
    onMenuZoomOut(() => step(-1)),
    onMenuZoomReset(() => apply(100)),
  ]);
  apply(100);
}
