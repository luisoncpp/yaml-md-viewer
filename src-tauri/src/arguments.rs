use crate::error::AppError;
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

pub fn parse_document_argument<I>(arguments: I, cwd: &Path) -> Result<Option<PathBuf>, AppError>
where
    I: IntoIterator<Item = OsString>,
{
    let mut values = arguments.into_iter();
    values.next();
    let mut positional = None;
    let mut options_ended = false;
    for value in values {
        if !options_ended && value == "--" {
            options_ended = true;
            continue;
        }
        if !options_ended && value.to_string_lossy().starts_with('-') {
            return Err(AppError::invalid_arguments("Unknown command-line option."));
        }
        if positional.replace(value).is_some() {
            return Err(AppError::invalid_arguments(
                "Only one document path may be supplied.",
            ));
        }
    }
    Ok(positional.map(|value| {
        let path = PathBuf::from(value);
        if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_relative_and_double_dash() {
        let cwd = Path::new("C:/work");
        let result = parse_document_argument(
            [
                OsString::from("app"),
                OsString::from("--"),
                OsString::from("a file.yaml.md"),
            ],
            cwd,
        )
        .unwrap()
        .unwrap();
        assert_eq!(result, PathBuf::from("C:/work/a file.yaml.md"));
    }
    #[test]
    fn rejects_options_and_many_paths() {
        assert!(
            parse_document_argument(
                [OsString::from("app"), OsString::from("--bad")],
                Path::new(".")
            )
            .is_err()
        );
        assert!(
            parse_document_argument(
                [
                    OsString::from("app"),
                    OsString::from("a.yaml.md"),
                    OsString::from("b.yaml.md")
                ],
                Path::new(".")
            )
            .is_err()
        );
    }
}
