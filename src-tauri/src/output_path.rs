use std::ffi::{OsStr, OsString};
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::error::ConversionResult;

// Stops the search if a folder is somehow full of copies.
const MAX_NAME_ATTEMPTS: u32 = 10_000;

// Claims "<stem>.<ext>", or "<stem> (2).<ext>" and so on if taken, by creating
// the file. Creating it (not just checking) stops parallel conversions picking
// the same name. Never overwrites.
pub fn reserve_output_path(
    input_path: &Path,
    directory: &Path,
    extension: &str,
) -> ConversionResult<PathBuf> {
    let stem = input_path
        .file_stem()
        .unwrap_or_else(|| OsStr::new("converted"));

    for attempt in 1..=MAX_NAME_ATTEMPTS {
        let candidate_path = directory.join(candidate_file_name(stem, extension, attempt));
        let creation_result = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate_path);

        match creation_result {
            Ok(_) => return Ok(candidate_path),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }

    let exhausted_error = std::io::Error::new(
        ErrorKind::AlreadyExists,
        "Too many files with this name already exist",
    );
    Err(exhausted_error.into())
}

// Built as an OsString so names that aren't valid Unicode (allowed on Linux)
// are kept byte for byte.
fn candidate_file_name(stem: &OsStr, extension: &str, attempt: u32) -> OsString {
    let mut file_name = stem.to_os_string();
    if attempt > 1 {
        file_name.push(format!(" ({attempt})"));
    }
    file_name.push(".");
    file_name.push(extension);
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
        let second =
            reserve_output_path(&input_path, directory.path(), "jpg").expect("second name");

        assert_eq!(first.file_name().unwrap(), "holiday.jpg");
        assert_eq!(second.file_name().unwrap(), "holiday (2).jpg");
    }

    #[cfg(unix)]
    #[test]
    fn names_that_are_not_unicode_are_kept_byte_for_byte() {
        use std::os::unix::ffi::OsStrExt;

        let stem = OsStr::from_bytes(b"caf\xe9");

        let file_name = candidate_file_name(stem, "jpg", 2);

        assert_eq!(file_name.as_bytes(), b"caf\xe9 (2).jpg");
    }

    #[test]
    fn never_reuses_the_input_file_name() {
        let directory = tempfile::tempdir().expect("temp dir");
        let input_path = directory.path().join("clip.mp4");
        std::fs::write(&input_path, b"original").expect("write input");

        let output_path =
            reserve_output_path(&input_path, directory.path(), "mp4").expect("output name");

        assert_eq!(output_path.file_name().unwrap(), "clip (2).mp4");
        assert_eq!(std::fs::read(&input_path).unwrap(), b"original");
    }
}
