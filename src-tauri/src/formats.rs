use std::path::Path;

use serde::{Deserialize, Serialize};

// Every supported format. The UI is built from `ConversionCatalog`.

const IMAGE_INPUT_EXTENSIONS: &[&str] = &[
    "avif", "bmp", "gif", "heic", "heif", "jpeg", "jpg", "pdf", "png", "svg", "tif", "tiff", "webp",
];

const VIDEO_INPUT_EXTENSIONS: &[&str] = &[
    "3gp", "avi", "flv", "m2ts", "m4v", "mkv", "mov", "mp4", "mpeg", "mpg", "mts", "ogv", "ts",
    "webm", "wmv",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Video,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    Jpg,
    Png,
    Webp,
    Avif,
    Heic,
    Gif,
    Bmp,
    Tiff,
    Pdf,
    Svg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoFormat {
    Mp4,
    Mov,
    Mkv,
    Webm,
    M4v,
    Avi,
    Flv,
    Gif,
}

impl ImageFormat {
    const ALL: [ImageFormat; 10] = [
        ImageFormat::Jpg,
        ImageFormat::Png,
        ImageFormat::Webp,
        ImageFormat::Avif,
        ImageFormat::Heic,
        ImageFormat::Gif,
        ImageFormat::Bmp,
        ImageFormat::Tiff,
        ImageFormat::Pdf,
        ImageFormat::Svg,
    ];

    pub fn extension(self) -> &'static str {
        match self {
            ImageFormat::Jpg => "jpg",
            ImageFormat::Png => "png",
            ImageFormat::Webp => "webp",
            ImageFormat::Avif => "avif",
            ImageFormat::Heic => "heic",
            ImageFormat::Gif => "gif",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Tiff => "tiff",
            ImageFormat::Pdf => "pdf",
            ImageFormat::Svg => "svg",
        }
    }

    fn label(self) -> &'static str {
        match self {
            ImageFormat::Jpg => "JPG",
            ImageFormat::Png => "PNG",
            ImageFormat::Webp => "WebP",
            ImageFormat::Avif => "AVIF",
            ImageFormat::Heic => "HEIC",
            ImageFormat::Gif => "GIF",
            ImageFormat::Bmp => "BMP",
            ImageFormat::Tiff => "TIFF",
            ImageFormat::Pdf => "PDF",
            ImageFormat::Svg => "SVG",
        }
    }

    fn description(self) -> &'static str {
        match self {
            ImageFormat::Jpg => "Photos. Opens anywhere. No transparency.",
            ImageFormat::Png => "Lossless. Keeps transparency.",
            ImageFormat::Webp => "Smaller than JPG. Keeps transparency.",
            ImageFormat::Avif => "Smallest files. Modern browsers only.",
            ImageFormat::Heic => "Apple photo format.",
            ImageFormat::Gif => "256 colours. Simple graphics.",
            ImageFormat::Bmp => "Uncompressed. Large files.",
            ImageFormat::Tiff => "Lossless. Print and archiving.",
            ImageFormat::Pdf => "One page per image.",
            ImageFormat::Svg => "Traced to vector shapes. Logos and icons.",
        }
    }

    pub fn uses_quality(self) -> bool {
        matches!(
            self,
            ImageFormat::Jpg
                | ImageFormat::Webp
                | ImageFormat::Avif
                | ImageFormat::Heic
                | ImageFormat::Pdf
        )
    }

    // HEIC is written with macOS `sips`; there's no encoder to bundle elsewhere.
    fn is_available_on_this_platform(self) -> bool {
        let is_unsupported_heic = self == ImageFormat::Heic && !cfg!(target_os = "macos");
        !is_unsupported_heic
    }
}

impl VideoFormat {
    const ALL: [VideoFormat; 8] = [
        VideoFormat::Mp4,
        VideoFormat::Mov,
        VideoFormat::Mkv,
        VideoFormat::Webm,
        VideoFormat::M4v,
        VideoFormat::Avi,
        VideoFormat::Flv,
        VideoFormat::Gif,
    ];

    pub fn extension(self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "mp4",
            VideoFormat::Mov => "mov",
            VideoFormat::Mkv => "mkv",
            VideoFormat::Webm => "webm",
            VideoFormat::M4v => "m4v",
            VideoFormat::Avi => "avi",
            VideoFormat::Flv => "flv",
            VideoFormat::Gif => "gif",
        }
    }

    fn label(self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "MP4",
            VideoFormat::Mov => "MOV",
            VideoFormat::Mkv => "MKV",
            VideoFormat::Webm => "WebM",
            VideoFormat::M4v => "M4V",
            VideoFormat::Avi => "AVI",
            VideoFormat::Flv => "FLV",
            VideoFormat::Gif => "GIF",
        }
    }

    fn description(self) -> &'static str {
        match self {
            VideoFormat::Mp4 => "Plays almost everywhere.",
            VideoFormat::Mov => "QuickTime. Common on Mac.",
            VideoFormat::Mkv => "Flexible container.",
            VideoFormat::Webm => "Open web format. Small files.",
            VideoFormat::M4v => "Apple's MP4 variant.",
            VideoFormat::Avi => "Legacy Windows format.",
            VideoFormat::Flv => "Legacy Flash video.",
            VideoFormat::Gif => "Silent animation. Keep clips short.",
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatOption<Format> {
    id: Format,
    label: &'static str,
    description: &'static str,
    uses_quality: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionCatalog {
    platform: &'static str,
    image_outputs: Vec<FormatOption<ImageFormat>>,
    video_outputs: Vec<FormatOption<VideoFormat>>,
    image_input_extensions: &'static [&'static str],
    video_input_extensions: &'static [&'static str],
}

pub fn build_catalog() -> ConversionCatalog {
    let image_outputs = ImageFormat::ALL
        .into_iter()
        .filter(|format| format.is_available_on_this_platform())
        .map(|format| FormatOption {
            id: format,
            label: format.label(),
            description: format.description(),
            uses_quality: format.uses_quality(),
        })
        .collect();

    let video_outputs = VideoFormat::ALL
        .into_iter()
        .map(|format| FormatOption {
            id: format,
            label: format.label(),
            description: format.description(),
            uses_quality: true,
        })
        .collect();

    let catalog = ConversionCatalog {
        platform: std::env::consts::OS,
        image_outputs,
        video_outputs,
        image_input_extensions: IMAGE_INPUT_EXTENSIONS,
        video_input_extensions: VIDEO_INPUT_EXTENSIONS,
    };
    catalog
}

pub fn lowercase_extension(path: &Path) -> String {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    extension
}

pub fn classify_path(path: &Path) -> Option<MediaKind> {
    let extension = lowercase_extension(path);

    if IMAGE_INPUT_EXTENSIONS.contains(&extension.as_str()) {
        return Some(MediaKind::Image);
    }
    if VIDEO_INPUT_EXTENSIONS.contains(&extension.as_str()) {
        return Some(MediaKind::Video);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_by_extension_case_insensitively() {
        assert_eq!(
            classify_path(Path::new("/photos/IMG_0001.HEIC")),
            Some(MediaKind::Image)
        );
        assert_eq!(classify_path(Path::new("clip.MoV")), Some(MediaKind::Video));
        assert_eq!(classify_path(Path::new("notes.txt")), None);
        assert_eq!(classify_path(Path::new("no-extension")), None);
    }

    #[test]
    fn heic_output_is_only_offered_on_macos() {
        let catalog = build_catalog();
        let offers_heic = catalog
            .image_outputs
            .iter()
            .any(|option| option.id == ImageFormat::Heic);
        assert_eq!(offers_heic, cfg!(target_os = "macos"));
    }
}
