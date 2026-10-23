use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::ConversionResult;

// Gives up rather than looping forever if a folder is somehow full of copies.
const MAX_NAME_ATTEMPTS: u32 = 10_000;

// Picks "<stem>.<extension>" in `directory`, or "<stem> (2).<extension>",
// "<stem> (3).<extension>"… if that name is taken, and creates an empty file
// there to claim it. Creating the file (instead of only checking it exists)
// stops two conversions running in parallel — e.g. photo.png and photo.webp
// both becoming photo.jpg — from picking the same name. Never overwrites.
pub fn reserve_output_path(input_path: &Path, directory: &Path, extension: &str) -> ConversionResult<PathBuf> {
    let stem = input_path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| "converted".to_string());

    for attempt in 1..=MAX_NAME_ATTEMPTS {
        let candidate_path = directory.join(candidate_file_name(&stem, extension, attempt));
        let creation_result = OpenOptions::new().write(true).create_new(true).open(&candidate_path);

        match creation_result {
            Ok(_) => return Ok(candidate_path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }

    let exhausted_error = std::io::Error::new(ErrorKind::AlreadyExists, "Too many files with this name already exist");
    Err(exhausted_error.into())
}

fn candidate_file_name(stem: &str, extension: &str, attempt: u32) -> String {
    let file_name = if attempt == 1 {
        format!("{stem}.{extension}")
    } else {
        format!("{stem} ({attempt}).{extension}")
    };
    file_name
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_the_name_instead_of_overwriting() {
        let directory = tempfile::tempdir().expect("temp dir");
        let input_path = directory.path().join("holiday.png");

        let first = reserve_output_path(&input_path, directory.path(), "jpg").expect("first name");
        let second = reserve_output_path(&input_path, directory.path(), "jpg").expect("second name");

        assert_eq!(first.file_name().unwrap(), "holiday.jpg");
        assert_eq!(second.file_name().unwrap(), "holiday (2).jpg");
    }

    #[test]
    fn never_reuses_the_input_file_name() {
        let directory = tempfile::tempdir().expect("temp dir");
        let input_path = directory.path().join("clip.mp4");
        std::fs::write(&input_path, b"original").expect("write input");

        let output_path = reserve_output_path(&input_path, directory.path(), "mp4").expect("output name");

        assert_eq!(output_path.file_name().unwrap(), "clip (2).mp4");
        assert_eq!(std::fs::read(&input_path).unwrap(), b"original");
    }
}
