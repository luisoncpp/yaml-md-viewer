const previewPolicy = "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:; font-src data:; connect-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'; media-src 'none'";
export const previewKeyMessage = "yamlmdviewer-preview-key";
export const previewZoomMessage = "yamlmdviewer-preview-zoom";

const keyboardBridge = `<script>addEventListener("keydown",event=>{if(event.key==="F11"||event.key==="Escape"){event.preventDefault();parent.postMessage({type:"${previewKeyMessage}",key:event.key},"*");return}if(event.ctrlKey&&!event.altKey&&["+","=","-","0"].includes(event.key)){event.preventDefault();parent.postMessage({type:"${previewZoomMessage}",key:event.key},"*")}});addEventListener("wheel",event=>{if(event.ctrlKey){event.preventDefault();parent.postMessage({type:"${previewZoomMessage}",deltaY:event.deltaY},"*")}},{passive:false})</script>`;

function securedHtml(html: string): string {
  const shell = `<meta http-equiv="Content-Security-Policy" content="${previewPolicy}">${keyboardBridge}`;
  return /<head(?:\s[^>]*)?>/i.test(html) ? html.replace(/<head(?:\s[^>]*)?>/i, match => `${match}${shell}`) : `<!doctype html><html><head>${shell}</head><body>${html}</body></html>`;
}

export function showPreview(frame: HTMLIFrameElement, html: string): void { frame.srcdoc = securedHtml(html); }
export function clearPreview(frame: HTMLIFrameElement): void { frame.srcdoc = securedHtml(""); }
