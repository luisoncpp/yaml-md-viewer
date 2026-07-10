import { getFullscreen, onFullscreenChanged, setFullscreen } from "./api";
import { previewKeyMessage } from "./preview";

export function setupFullscreen(preview: HTMLIFrameElement): void {
  let fullscreen = false;
  let changing = false;

  const apply = (next: boolean) => {
    fullscreen = next;
    document.body.classList.toggle("fullscreen", next);
  };
  const sync = async () => apply(await getFullscreen());
  const change = async (next: boolean) => {
    if (changing) return;
    changing = true;
    try {
      apply(await setFullscreen(next));
    } catch {
      await sync();
    } finally {
      changing = false;
    }
  };
  const handleKey = (key: string, preventDefault: () => void) => {
    if (key === "F11") {
      preventDefault();
      void change(!fullscreen);
    } else if (key === "Escape" && fullscreen) {
      preventDefault();
      void change(false);
    }
  };

  window.addEventListener("keydown", event => {
    if (!event.repeat) handleKey(event.key, () => event.preventDefault());
  });
  window.addEventListener("message", event => {
    if (event.source !== preview.contentWindow || event.data?.type !== previewKeyMessage) return;
    handleKey(event.data.key, () => undefined);
  });
  window.addEventListener("resize", () => void sync());
  void onFullscreenChanged(apply);
  void sync();
}
