use std::path::PathBuf;

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

    #[error("Converting produced an empty file")]
    EmptyOutput,

    #[error("The bundled FFmpeg is missing (looked in {0}). Reinstall the app.")]
    MissingFfmpeg(PathBuf),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Image(#[from] image::ImageError),
}

pub type ConversionResult<T> = Result<T, ConversionError>;
