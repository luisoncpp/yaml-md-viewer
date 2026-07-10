import { open, save } from "@tauri-apps/plugin-dialog";
import {
  currentDocument,
  onDocumentError,
  onDocumentOpened,
  onMenuOpenDocument,
  onMenuSaveAsHtml,
  openDocument,
  saveDocument,
} from "./api";
import { defaultHtmlPath } from "./filenames";
import { setupFullscreen } from "./fullscreen";
import type { AppError, DocumentView } from "./models";
import { clearPreview, showPreview } from "./preview";
import { setupZoom } from "./zoom";
import "./styles.css";

const app = document.querySelector<HTMLDivElement>("#app")!;
app.innerHTML = `<iframe id="preview" title="Compiled document preview" sandbox="allow-scripts"></iframe><footer id="status-bar"><section id="status" role="status" aria-live="polite"></section><div class="document"><strong id="title">No document open</strong><span id="path"></span></div></footer>`;
const title = document.querySelector<HTMLElement>("#title")!;
const pathText = document.querySelector<HTMLElement>("#path")!;
const status = document.querySelector<HTMLElement>("#status")!;
const preview = document.querySelector<HTMLIFrameElement>("#preview")!;
clearPreview(preview);
let displayedRevision = 0;
let current: DocumentView | null = null;

function setStatus(message: string, error = false): void {
  status.textContent = message;
  status.classList.toggle("error", error);
}

function showDocument(document: DocumentView): void {
  if (document.revision < displayedRevision) return;
  displayedRevision = document.revision;
  current = document;
  title.textContent = document.displayTitle;
  pathText.textContent = document.sourcePath;
  pathText.title = document.sourcePath;
  showPreview(preview, document.revision);
  setStatus("Document loaded.");
}

function showError(error: AppError): void {
  setStatus(`${error.code}: ${error.message}`, true);
}

async function chooseDocument(): Promise<void> {
  const selected = await open({
    multiple: false,
    directory: false,
    filters: [{ name: "YAML Markdown", extensions: ["yaml.md"] }],
  });
  if (!selected || Array.isArray(selected)) return;
  setStatus("Loading document…");
  try {
    showDocument(await openDocument(selected));
  } catch (error) {
    showError(error as AppError);
  }
}

async function saveAsHtml(): Promise<void> {
  if (!current) return;
  const selected = await save({
    defaultPath: defaultHtmlPath(current.sourcePath),
    filters: [{ name: "HTML", extensions: ["html"] }],
  });
  if (!selected) return;
  try {
    const result = await saveDocument(selected);
    setStatus(`Saved to ${result.path}`);
  } catch (error) {
    showError(error as AppError);
  }
}

setupFullscreen(preview);
setupZoom(preview);
void Promise.all([
  onDocumentOpened(showDocument),
  onDocumentError(showError),
  onMenuOpenDocument(() => void chooseDocument()),
  onMenuSaveAsHtml(() => void saveAsHtml()),
]).then(async () => {
  try {
    const document = await currentDocument();
    if (document) showDocument(document);
  } catch (error) {
    showError(error as AppError);
  }
});
