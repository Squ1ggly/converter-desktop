use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("Conversion was cancelled")]
    Cancelled,

    #[error("This file could not be read as an image: {0}")]
    UnreadableImage(String),

    #[error("{0}")]
    Ffmpeg(String),

    #[error("Could not trace this image into an SVG: {0}")]
    Svg(String),

    #[error("macOS could not write the HEIC file: {0}")]
    Heic(String),

    #[error("This image is too large to convert")]
    ImageTooLarge,

    #[error("This image is too large to convert at that size. Choose a smaller size.")]
    TargetTooLarge,

    #[error("Only files with a full path can be converted")]
    RelativePath,

    #[error("Converting produced an empty file")]
    EmptyOutput,

    #[error("{file_name} is no longer there. It may have been moved or deleted.")]
    FileMissing { file_name: String },

    #[error("Can't read {file_name}. Check the file's permissions.")]
    FileNotReadable { file_name: String },

    #[error("The folder {folder_name} no longer exists")]
    FolderMissing { folder_name: String },

    #[error("Can't write to {folder_name}. Check the folder's permissions.")]
    FolderNotWritable { folder_name: String },

    #[error("There isn't enough space to save in {folder_name}")]
    FolderFull { folder_name: String },

    #[error("The conversion stopped unexpectedly. The log has the details.")]
    Unexpected,

    #[error("The bundled FFmpeg is missing (looked in {0}). Reinstall the app.")]
    MissingFfmpeg(PathBuf),

    #[error("{}", without_os_error_code(.0))]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Image(#[from] image::ImageError),
}

pub type ConversionResult<T> = Result<T, ConversionError>;

impl ConversionError {
    // For errors while reading `input_path`.
    #[must_use]
    pub fn about_input(self, input_path: &Path) -> ConversionError {
        let ConversionError::Io(io_error) = self else {
            return self;
        };
        let file_name = display_name(input_path);

        let described_error = match io_error.kind() {
            ErrorKind::NotFound => ConversionError::FileMissing { file_name },
            ErrorKind::PermissionDenied => ConversionError::FileNotReadable { file_name },
            _ => ConversionError::Io(io_error),
        };
        described_error
    }

    // For errors while finding or writing to `folder`.
    #[must_use]
    pub fn about_output_folder(self, folder: &Path) -> ConversionError {
        let ConversionError::Io(io_error) = self else {
            return self;
        };
        let folder_name = display_name(folder);

        let described_error = match io_error.kind() {
            ErrorKind::NotFound | ErrorKind::NotADirectory => {
                ConversionError::FolderMissing { folder_name }
            }
            ErrorKind::PermissionDenied | ErrorKind::ReadOnlyFilesystem => {
                ConversionError::FolderNotWritable { folder_name }
            }
            ErrorKind::StorageFull | ErrorKind::QuotaExceeded => {
                ConversionError::FolderFull { folder_name }
            }
            _ => ConversionError::Io(io_error),
        };
        described_error
    }

    // For errors during a conversion. The output file already exists by then,
    // so a missing file or a refused read is about the input, and running out
    // of space is about the output.
    #[must_use]
    pub fn about_conversion(self, input_path: &Path, output_folder: &Path) -> ConversionError {
        let is_about_output = matches!(
            &self,
            ConversionError::Io(io_error) if matches!(
                io_error.kind(),
                ErrorKind::StorageFull | ErrorKind::QuotaExceeded | ErrorKind::ReadOnlyFilesystem
            )
        );
        if is_about_output {
            return self.about_output_folder(output_folder);
        }
        self.about_input(input_path)
    }
}

// The last part of the path, such as "Desktop", or the whole path for a drive.
fn display_name(path: &Path) -> String {
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |file_name| file_name.to_string_lossy().into_owned(),
    );
    name
}

// "Permission denied (os error 13)" becomes "Permission denied".
fn without_os_error_code(error: &std::io::Error) -> String {
    let message = error.to_string();
    let Some(code_start) = message.rfind(" (os error ") else {
        return message;
    };
    let message_without_code = message[..code_start].to_string();
    message_without_code
}

#[cfg(test)]
mod tests {
    use super::*;

    fn io_error(kind: ErrorKind) -> ConversionError {
        ConversionError::Io(std::io::Error::from(kind))
    }

    #[test]
    fn folder_errors_name_the_folder() {
        let desktop = Path::new("/Users/sam/Desktop");

        let refused = io_error(ErrorKind::PermissionDenied).about_output_folder(desktop);
        let missing = io_error(ErrorKind::NotFound).about_output_folder(desktop);
        let full = io_error(ErrorKind::StorageFull).about_output_folder(desktop);

        assert_eq!(
            refused.to_string(),
            "Can't write to Desktop. Check the folder's permissions."
        );
        assert_eq!(missing.to_string(), "The folder Desktop no longer exists");
        assert_eq!(
            full.to_string(),
            "There isn't enough space to save in Desktop"
        );
    }

    #[test]
    fn input_errors_name_the_file() {
        let photo = Path::new("/Users/sam/Pictures/photo.png");

        let missing = io_error(ErrorKind::NotFound).about_input(photo);
        let refused = io_error(ErrorKind::PermissionDenied).about_input(photo);

        assert_eq!(
            missing.to_string(),
            "photo.png is no longer there. It may have been moved or deleted."
        );
        assert_eq!(
            refused.to_string(),
            "Can't read photo.png. Check the file's permissions."
        );
    }

    #[test]
    fn other_operating_system_errors_drop_the_error_code() {
        let raw_error = ConversionError::Io(std::io::Error::from_raw_os_error(5));

        let message = raw_error.to_string();

        assert!(!message.contains("os error"), "{message}");
        assert!(!message.is_empty());
    }
}
