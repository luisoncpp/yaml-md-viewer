use serde::Serialize;
use std::path::PathBuf;

#[derive(Clone)]
pub struct DocumentSnapshot {
    pub revision: u64,
    pub source_path: PathBuf,
    pub display_title: String,
    pub compiled_html: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentView {
    pub revision: u64,
    pub source_path: String,
    pub display_title: String,
    pub compiled_html: String,
}

impl From<&DocumentSnapshot> for DocumentView {
    fn from(value: &DocumentSnapshot) -> Self {
        Self {
            revision: value.revision,
            source_path: value.source_path.to_string_lossy().into_owned(),
            display_title: value.display_title.clone(),
            compiled_html: value.compiled_html.clone(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResult {
    pub path: String,
}

#[derive(Default)]
pub struct AppState {
    pub next_revision: u64,
    pub current: Option<DocumentSnapshot>,
}
