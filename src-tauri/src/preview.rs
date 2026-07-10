use crate::models::AppState;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, Runtime, http};

const POLICY: &str = "default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data:; font-src data:; connect-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'; media-src 'none'";
const BRIDGE: &str = r#"<script>addEventListener("keydown",event=>{if(event.key==="F11"||event.key==="Escape"){event.preventDefault();parent.postMessage({type:"yamlmdviewer-preview-key",key:event.key},"*");return}if(event.ctrlKey&&!event.altKey&&["+","=","-","0"].includes(event.key)){event.preventDefault();parent.postMessage({type:"yamlmdviewer-preview-zoom",key:event.key},"*")}});addEventListener("wheel",event=>{if(event.ctrlKey){event.preventDefault();parent.postMessage({type:"yamlmdviewer-preview-zoom",deltaY:event.deltaY},"*")}},{passive:false});addEventListener("message",event=>{if(event.source===parent&&event.data?.type==="yamlmdviewer-preview-apply-zoom"&&Number.isFinite(event.data.percentage)){document.documentElement.style.zoom=event.data.percentage/100}})</script>"#;

pub fn respond<R: Runtime>(app: &AppHandle<R>, path: &str) -> http::Response<Vec<u8>> {
    let mut path_parts = path.trim_matches('/').split('/');
    let requested_revision = path_parts.next().and_then(|part| part.parse::<u64>().ok());
    let dark_mode = path_parts.next() == Some("dark");
    let html = app
        .state::<Mutex<AppState>>()
        .lock()
        .ok()
        .and_then(|state| {
            state.current.as_ref().and_then(|document| {
                (Some(document.revision) == requested_revision)
                    .then(|| document.compiled_html.clone())
            })
        })
        .unwrap_or_default();

    let body = secured_html(&html, dark_mode).into_bytes();
    http::Response::builder()
        .header(http::header::CONTENT_TYPE, "text/html; charset=utf-8")
        .header(http::header::CONTENT_SECURITY_POLICY, POLICY)
        .header(http::header::CACHE_CONTROL, "no-store")
        .header("X-Content-Type-Options", "nosniff")
        .body(body)
        .expect("static preview response headers are valid")
}

fn secured_html(html: &str, dark_mode: bool) -> String {
    let html = apply_color_mode(html, dark_mode);
    let head = html
        .to_ascii_lowercase()
        .find("<head")
        .and_then(|start| html[start..].find('>').map(|end| start + end + 1));

    if let Some(index) = head {
        let mut secured = String::with_capacity(html.len() + BRIDGE.len());
        secured.push_str(&html[..index]);
        secured.push_str(BRIDGE);
        secured.push_str(&html[index..]);
        secured
    } else {
        let theme = if dark_mode {
            " data-theme=\"dark\""
        } else {
            ""
        };
        format!("<!doctype html><html{theme}><head>{BRIDGE}</head><body>{html}</body></html>")
    }
}

fn apply_color_mode(html: &str, dark_mode: bool) -> String {
    if !dark_mode {
        return html.to_owned();
    }

    let Some(start) = html.to_ascii_lowercase().find("<html") else {
        return html.to_owned();
    };
    let Some(relative_end) = html[start..].find('>') else {
        return html.to_owned();
    };
    let end = start + relative_end;
    if html[start..=end]
        .to_ascii_lowercase()
        .contains("data-theme=")
    {
        return html.to_owned();
    }

    let mut themed = String::with_capacity(html.len() + 18);
    themed.push_str(&html[..end]);
    themed.push_str(" data-theme=\"dark\"");
    themed.push_str(&html[end..]);
    themed
}

#[cfg(test)]
mod tests {
    use super::{BRIDGE, secured_html};

    #[test]
    fn injects_bridge_into_existing_head_without_changing_body() {
        let compiled =
            "<!doctype html><html><head><title>Example</title></head><body>Body</body></html>";
        let secured = secured_html(compiled, false);

        assert!(secured.contains(BRIDGE));
        assert!(secured.contains("<body>Body</body>"));
        assert!(secured.find(BRIDGE) < secured.find("<title>"));
    }

    #[test]
    fn wraps_fragments() {
        let secured = secured_html("<main>Body</main>", false);

        assert!(secured.starts_with("<!doctype html><html><head>"));
        assert!(secured.contains(BRIDGE));
        assert!(secured.ends_with("<body><main>Body</main></body></html>"));
    }

    #[test]
    fn applies_dark_mode_before_the_document_head_is_loaded() {
        let compiled = "<!doctype html><html lang=\"en\"><head></head><body>Body</body></html>";

        let secured = secured_html(compiled, true);

        assert!(secured.contains("<html lang=\"en\" data-theme=\"dark\">"));
    }

    #[test]
    fn bridge_applies_zoom_inside_the_preview_document() {
        assert!(BRIDGE.contains("document.documentElement.style.zoom"));
        assert!(BRIDGE.contains("event.source===parent"));
    }

    #[test]
    fn leaves_light_documents_without_a_theme_override() {
        let compiled = "<!doctype html><html lang=\"en\"><head></head><body>Body</body></html>";

        let secured = secured_html(compiled, false);

        assert!(secured.contains("<html lang=\"en\">"));
        assert!(!secured.contains("data-theme=\"dark\""));
    }
}
