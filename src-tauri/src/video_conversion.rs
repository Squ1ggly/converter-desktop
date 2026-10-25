use std::ffi::OsString;
use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::error::ConversionResult;
use crate::ffmpeg::Ffmpeg;
use crate::formats::VideoFormat;
use crate::options::{VideoOptions, VideoQuality};

const DEFAULT_GIF_FRAME_RATE: u32 = 15;

pub struct VideoConversion<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub format: VideoFormat,
    pub options: VideoOptions,
    pub ffmpeg: &'a Ffmpeg,
    pub cancellation: &'a CancellationToken,
}

pub async fn convert_video(
    conversion: VideoConversion<'_>,
    on_progress: impl FnMut(f32),
) -> ConversionResult<()> {
    let arguments = build_arguments(
        conversion.input_path,
        conversion.output_path,
        conversion.format,
        conversion.options,
    );
    conversion
        .ffmpeg
        .run(arguments, conversion.cancellation, on_progress)
        .await
}

fn build_arguments(
    input_path: &Path,
    output_path: &Path,
    format: VideoFormat,
    options: VideoOptions,
) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = vec!["-i".into(), input_path.into()];

    if format == VideoFormat::Gif {
        arguments.extend(gif_arguments(options));
    } else {
        arguments.extend(["-vf".into(), scale_filter(options.max_height).into()]);
        arguments.extend(frame_rate_arguments(options.frame_rate));
        arguments.extend(codec_arguments(format, options.quality));
    }

    arguments.extend([
        "-f".into(),
        format.ffmpeg_muxer().into(),
        output_path.into(),
    ]);
    arguments
}

// The limit applies to the shorter side, so "720p" works for portrait video
// too. Never enlarges. yuv420p needs even dimensions.
fn scale_filter(max_height: Option<u32>) -> String {
    let Some(limit) = max_height else {
        return "scale='trunc(iw/2)*2':'trunc(ih/2)*2'".to_string();
    };

    let filter = format!(
        "scale=w='if(gte(iw,ih),-2,trunc(min(iw,{limit})/2)*2)':h='if(gte(iw,ih),trunc(min(ih,{limit})/2)*2,-2)'"
    );
    filter
}

// Only ever lowers the frame rate.
fn frame_rate_arguments(frame_rate: Option<u32>) -> Vec<OsString> {
    let Some(frame_rate) = frame_rate else {
        return Vec::new();
    };

    let arguments = vec!["-fpsmax".into(), frame_rate.to_string().into()];
    arguments
}

// A palette built from the clip looks far better than a generic one.
fn gif_arguments(options: VideoOptions) -> Vec<OsString> {
    let frame_rate = options.frame_rate.unwrap_or(DEFAULT_GIF_FRAME_RATE);
    let scale = options
        .max_height
        .map_or("scale=iw:ih".to_string(), |limit| {
            format!(
                "scale=w='if(gte(iw,ih),-1,min(iw,{limit}))':h='if(gte(iw,ih),min(ih,{limit}),-1)'"
            )
        });
    let filter = format!(
        "fps=fps='min(source_fps,{frame_rate})',{scale}:flags=lanczos,split[frames][palette_source];\
         [palette_source]palettegen[palette];[frames][palette]paletteuse"
    );

    let arguments = vec!["-filter_complex".into(), filter.into(), "-an".into()];
    arguments
}

fn codec_arguments(format: VideoFormat, quality: VideoQuality) -> Vec<OsString> {
    let codec_flags: Vec<String> = match format {
        VideoFormat::Webm => webm_codec_flags(quality),
        VideoFormat::Avi => avi_codec_flags(quality),
        VideoFormat::Mp4 | VideoFormat::M4v | VideoFormat::Mov => {
            let mut flags = h264_codec_flags(quality);
            // Lets playback start before the whole file is read.
            flags.extend(["-movflags".to_string(), "+faststart".to_string()]);
            flags
        }
        VideoFormat::Mkv | VideoFormat::Flv | VideoFormat::Gif => h264_codec_flags(quality),
    };

    let arguments = codec_flags.into_iter().map(OsString::from).collect();
    arguments
}

fn h264_codec_flags(quality: VideoQuality) -> Vec<String> {
    let (crf, audio_bitrate) = match quality {
        VideoQuality::High => ("18", "192k"),
        VideoQuality::Balanced => ("23", "160k"),
        VideoQuality::Small => ("28", "128k"),
    };

    let flags = [
        "-c:v",
        "libx264",
        "-preset",
        "medium",
        "-crf",
        crf,
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "aac",
        "-b:a",
        audio_bitrate,
    ];
    flags.map(String::from).to_vec()
}

fn webm_codec_flags(quality: VideoQuality) -> Vec<String> {
    let (crf, audio_bitrate) = match quality {
        VideoQuality::High => ("28", "160k"),
        VideoQuality::Balanced => ("33", "128k"),
        VideoQuality::Small => ("38", "96k"),
    };

    // "-b:v 0" switches VP9 into constant-quality mode so the CRF is honoured.
    let flags = [
        "-c:v",
        "libvpx-vp9",
        "-crf",
        crf,
        "-b:v",
        "0",
        "-row-mt",
        "1",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "libopus",
        "-b:a",
        audio_bitrate,
    ];
    flags.map(String::from).to_vec()
}

fn avi_codec_flags(quality: VideoQuality) -> Vec<String> {
    let (quantizer, audio_bitrate) = match quality {
        VideoQuality::High => ("2", "192k"),
        VideoQuality::Balanced => ("4", "160k"),
        VideoQuality::Small => ("7", "128k"),
    };

    let flags = [
        "-c:v",
        "mpeg4",
        "-q:v",
        quantizer,
        "-c:a",
        "libmp3lame",
        "-b:a",
        audio_bitrate,
    ];
    flags.map(String::from).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arguments_as_text(format: VideoFormat, options: VideoOptions) -> Vec<String> {
        build_arguments(Path::new("in.mov"), Path::new("out.file"), format, options)
            .into_iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    const BALANCED: VideoOptions = VideoOptions {
        quality: VideoQuality::Balanced,
        max_height: None,
        frame_rate: None,
    };

    #[test]
    fn mp4_uses_h264_with_fast_start() {
        let arguments = arguments_as_text(VideoFormat::Mp4, BALANCED);
        assert!(arguments.contains(&"libx264".to_string()));
        assert!(arguments.contains(&"+faststart".to_string()));
        assert_eq!(arguments.last().unwrap(), "out.file");
    }

    #[test]
    fn every_format_names_its_muxer() {
        for format in [VideoFormat::M4v, VideoFormat::Gif] {
            let arguments = arguments_as_text(format, BALANCED);
            let muxer_position = arguments.len() - 2;
            assert_eq!(arguments[muxer_position - 1], "-f");
            assert_eq!(arguments[muxer_position], format.ffmpeg_muxer());
        }
    }

    #[test]
    fn resolution_limit_only_shrinks() {
        let options = VideoOptions {
            max_height: Some(720),
            ..BALANCED
        };
        let arguments = arguments_as_text(VideoFormat::Mkv, options);
        let scale_filter = arguments
            .iter()
            .find(|argument| argument.starts_with("scale="))
            .expect("has a scale filter");
        assert!(scale_filter.contains("min(iw,720)") && scale_filter.contains("min(ih,720)"));
    }

    #[test]
    fn frame_rate_is_a_maximum() {
        let options = VideoOptions {
            frame_rate: Some(30),
            ..BALANCED
        };
        let arguments = arguments_as_text(VideoFormat::Mp4, options);
        assert!(arguments.windows(2).any(|pair| pair == ["-fpsmax", "30"]));
    }

    #[test]
    fn gif_drops_audio_and_builds_a_palette() {
        let arguments = arguments_as_text(VideoFormat::Gif, BALANCED);
        assert!(arguments.contains(&"-an".to_string()));
        assert!(
            arguments
                .iter()
                .any(|argument| argument.contains("palettegen"))
        );
    }
}
