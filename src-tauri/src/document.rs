use crate::{error::AppError, models::DocumentSnapshot};
use mdyaml2html::{CompileOptions, HtmlOptions, compile};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct DocumentService;

impl DocumentService {
    pub fn compile_path(path: &Path) -> Result<DocumentSnapshot, AppError> {
        Self::validate_input(path)?;
        let bytes = fs::read(path).map_err(AppError::from_read)?;
        let source = String::from_utf8(bytes)
            .map_err(|_| AppError::new("invalid_utf8", "The document must be encoded as UTF-8."))?;
        let options = CompileOptions {
            html: Some(HtmlOptions {
                enable_custom_scripts: Some(false),
                ..Default::default()
            }),
        };
        let compiled = compile(&source, &options).map_err(|error| {
            eprintln!("Document compilation failed: {error:#}");
            AppError::new(
                "compile_failed",
                "The YAML Markdown document could not be compiled.",
            )
        })?;
        let title = compiled
            .metadatata
            .and_then(|metadata| metadata.title)
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| Self::filename_title(path));
        Ok(DocumentSnapshot {
            revision: 0,
            source_path: path.to_path_buf(),
            display_title: title,
            compiled_html: compiled.html,
        })
    }

    pub fn validate_input(path: &Path) -> Result<(), AppError> {
        if !has_yaml_md_suffix(path) {
            return Err(AppError::new(
                "invalid_extension",
                "Choose a file ending in .yaml.md.",
            ));
        }
        let metadata = fs::metadata(path).map_err(AppError::from_read)?;
        if !metadata.is_file() {
            return Err(AppError::new(
                "not_a_file",
                "The selected path is not a regular file.",
            ));
        }
        Ok(())
    }

    pub fn export(path: &Path, html: &str) -> Result<PathBuf, AppError> {
        let destination = html_path(path);
        fs::write(&destination, html).map_err(|error| {
            eprintln!("Document export failed: {error}");
            AppError::new("write_failed", "The HTML file could not be written.")
        })?;
        Ok(destination)
    }

    fn filename_title(path: &Path) -> String {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| name[..name.len().saturating_sub(8)].to_owned())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| "Untitled document".to_owned())
    }
}

pub fn has_yaml_md_suffix(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().ends_with(".yaml.md"))
}
pub fn html_path(path: &Path) -> PathBuf {
    if path
        .to_string_lossy()
        .to_ascii_lowercase()
        .ends_with(".html")
    {
        path.to_path_buf()
    } else {
        PathBuf::from(format!("{}.html", path.to_string_lossy()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extensions_are_normalized() {
        assert!(has_yaml_md_suffix(Path::new("A.YAML.MD")));
        assert_eq!(
            html_path(Path::new("report")).to_string_lossy(),
            "report.html"
        );
        assert_eq!(
            html_path(Path::new("report.HTML")).to_string_lossy(),
            "report.HTML"
        );
    }
    #[test]
    fn custom_scripts_are_removed_but_assets_remain() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/custom-script.yaml.md");
        let document = DocumentService::compile_path(&path).unwrap();
        assert!(!document.compiled_html.contains("sourceScriptWasRun"));
        assert!(document.compiled_html.contains("querySelector"));
    }

    #[test]
    fn compiler_uses_metadata_title() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/valid.yaml.md");
        let document = DocumentService::compile_path(&path).unwrap();
        assert_eq!(document.display_title, "Valid fixture");
    }
}
