use serde::Serialize;
use std::io;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppError {
    pub code: &'static str,
    pub message: String,
}

impl AppError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn invalid_arguments(message: impl Into<String>) -> Self {
        Self::new("invalid_arguments", message)
    }

    pub fn from_read(error: io::Error) -> Self {
        let code = match error.kind() {
            io::ErrorKind::NotFound => "file_not_found",
            _ => "read_failed",
        };
        Self::new(
            code,
            if code == "file_not_found" {
                "The selected document no longer exists."
            } else {
                "The document could not be read."
            },
        )
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for AppError {}
