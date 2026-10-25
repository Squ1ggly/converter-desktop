use serde::Deserialize;

use crate::formats::{ImageFormat, VideoFormat};

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(
    tag = "mode",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ResizeOption {
    Original,
    Percent {
        percent: u32,
    },
    Fit {
        max_width: Option<u32>,
        max_height: Option<u32>,
    },
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOptions {
    // 1–100; only read by formats where `ImageFormat::uses_quality` is true.
    pub quality: u8,
    pub resize: ResizeOption,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VideoQuality {
    High,
    Balanced,
    Small,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoOptions {
    pub quality: VideoQuality,
    // Videos taller than this are scaled down; shorter ones are never enlarged.
    pub max_height: Option<u32>,
    pub frame_rate: Option<u32>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ConversionSettings {
    Image {
        format: ImageFormat,
        options: ImageOptions,
    },
    Video {
        format: VideoFormat,
        options: VideoOptions,
    },
}

impl ConversionSettings {
    pub fn output_extension(&self) -> &'static str {
        match self {
            ConversionSettings::Image { format, .. } => format.extension(),
            ConversionSettings::Video { format, .. } => format.extension(),
        }
    }

    // A video encode already uses every core, so videos run one at a time.
    pub fn max_parallel_files(&self) -> usize {
        match self {
            ConversionSettings::Image { .. } => parallel_image_limit(),
            ConversionSettings::Video { .. } => 1,
        }
    }
}

const MAX_PARALLEL_IMAGES: usize = 4;

fn parallel_image_limit() -> usize {
    let available_cores = std::thread::available_parallelism().map_or(1, std::num::NonZero::get);
    let limit = available_cores.clamp(1, MAX_PARALLEL_IMAGES);
    limit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_settings_sent_by_the_frontend() {
        let json = r#"{
            "kind": "image",
            "format": "webp",
            "options": { "quality": 80, "resize": { "mode": "fit", "maxWidth": 1200, "maxHeight": null } }
        }"#;
        let settings: ConversionSettings = serde_json::from_str(json).expect("valid settings");

        let ConversionSettings::Image { format, options } = settings else {
            panic!("expected image settings");
        };
        assert_eq!(format, ImageFormat::Webp);
        assert!(matches!(
            options.resize,
            ResizeOption::Fit {
                max_width: Some(1200),
                max_height: None
            }
        ));
    }
}
