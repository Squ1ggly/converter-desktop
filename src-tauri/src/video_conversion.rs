use std::ffi::OsString;
use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::error::ConversionResult;
use crate::ffmpeg::Ffmpeg;
use crate::formats::VideoFormat;
use crate::options::{VideoOptions, VideoQuality};

const DEFAULT_GIF_FRAME_RATE: u32 = 15;
// GIF stores every pixel of every frame, so larger sizes make huge files.
const MAX_GIF_SHORTER_SIDE: u32 = 480;
// The palette pass reports the first half of the progress, encoding the rest.
const GIF_PALETTE_PASS_SHARE: f32 = 0.5;

pub struct VideoConversion<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub format: VideoFormat,
    pub options: VideoOptions,
    pub ffmpeg: &'a Ffmpeg,
    pub scratch_root: &'a Path,
    pub cancellation: &'a CancellationToken,
}

pub async fn convert_video(
    conversion: VideoConversion<'_>,
    on_progress: impl FnMut(f32),
) -> ConversionResult<()> {
    if conversion.format == VideoFormat::Gif {
        return convert_to_gif(&conversion, on_progress).await;
    }

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
    arguments.extend(["-vf".into(), scale_filter(options.max_height).into()]);
    arguments.extend(frame_rate_arguments(options.frame_rate));
    arguments.extend(codec_arguments(format, options.quality));
    arguments.extend([
        "-f".into(),
        format.ffmpeg_muxer().into(),
        output_path.into(),
    ]);
    arguments
}

// Two passes: building the palette in the same run that uses it makes FFmpeg
// keep every frame in memory until the clip ends.
async fn convert_to_gif(
    conversion: &VideoConversion<'_>,
    mut on_progress: impl FnMut(f32),
) -> ConversionResult<()> {
    let scratch_directory = tempfile::tempdir_in(conversion.scratch_root)?;
    let palette_path = scratch_directory.path().join("palette.png");
    let frames_filter = gif_frames_filter(conversion.options);

    let palette_arguments =
        gif_palette_arguments(conversion.input_path, &frames_filter, &palette_path);
    let report_palette_progress = |percent: f32| on_progress(percent * GIF_PALETTE_PASS_SHARE);
    conversion
        .ffmpeg
        .run(
            palette_arguments,
            conversion.cancellation,
            report_palette_progress,
        )
        .await?;

    let encode_arguments = gif_encode_arguments(
        conversion.input_path,
        &palette_path,
        &frames_filter,
        conversion.output_path,
    );
    let report_encode_progress = |percent: f32| {
        let palette_pass_percent = 100.0 * GIF_PALETTE_PASS_SHARE;
        on_progress(palette_pass_percent + percent * (1.0 - GIF_PALETTE_PASS_SHARE));
    };
    conversion
        .ffmpeg
        .run(
            encode_arguments,
            conversion.cancellation,
            report_encode_progress,
        )
        .await
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

// Frame rate and size, shared by both GIF passes. The shorter side is capped
// at MAX_GIF_SHORTER_SIDE, or the chosen resolution if that is lower.
fn gif_frames_filter(options: VideoOptions) -> String {
    let frame_rate = options.frame_rate.unwrap_or(DEFAULT_GIF_FRAME_RATE);
    let shorter_side_limit = options.max_height.map_or(MAX_GIF_SHORTER_SIDE, |limit| {
        limit.min(MAX_GIF_SHORTER_SIDE)
    });

    let filter = format!(
        "fps=fps='min(source_fps,{frame_rate})',\
         scale=w='if(gte(iw,ih),-1,min(iw,{shorter_side_limit}))':\
         h='if(gte(iw,ih),min(ih,{shorter_side_limit}),-1)':flags=lanczos,setsar=1"
    );
    filter
}

// A palette built from the clip looks far better than a generic one.
fn gif_palette_arguments(
    input_path: &Path,
    frames_filter: &str,
    palette_path: &Path,
) -> Vec<OsString> {
    let arguments = vec![
        "-i".into(),
        input_path.into(),
        "-vf".into(),
        format!("{frames_filter},palettegen").into(),
        "-an".into(),
        "-f".into(),
        "image2".into(),
        "-update".into(),
        "1".into(),
        palette_path.into(),
    ];
    arguments
}

fn gif_encode_arguments(
    input_path: &Path,
    palette_path: &Path,
    frames_filter: &str,
    output_path: &Path,
) -> Vec<OsString> {
    let arguments = vec![
        "-i".into(),
        input_path.into(),
        "-i".into(),
        palette_path.into(),
        "-filter_complex".into(),
        format!("[0:v]{frames_filter}[frames];[frames][1:v]paletteuse").into(),
        "-an".into(),
        "-f".into(),
        VideoFormat::Gif.ffmpeg_muxer().into(),
        output_path.into(),
    ];
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
        VideoFormat::Mkv | VideoFormat::Flv => h264_codec_flags(quality),
        // GIF has no quality options; see convert_to_gif.
        VideoFormat::Gif => Vec::new(),
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

    fn as_text(arguments: Vec<OsString>) -> Vec<String> {
        arguments
            .into_iter()
            .map(|argument| argument.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn the_muxer_is_named_instead_of_guessed_from_the_extension() {
        let arguments = arguments_as_text(VideoFormat::M4v, BALANCED);
        assert!(
            arguments
                .windows(3)
                .any(|triple| triple == ["-f", "ipod", "out.file"])
        );
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
    fn gif_builds_the_palette_in_its_own_pass() {
        let frames_filter = gif_frames_filter(BALANCED);
        let palette_pass = as_text(gif_palette_arguments(
            Path::new("in.mov"),
            &frames_filter,
            Path::new("palette.png"),
        ));
        let encode_pass = as_text(gif_encode_arguments(
            Path::new("in.mov"),
            Path::new("palette.png"),
            &frames_filter,
            Path::new("out.gif"),
        ));

        assert!(
            palette_pass
                .iter()
                .any(|argument| argument.ends_with(",palettegen"))
        );
        assert!(
            encode_pass
                .iter()
                .any(|argument| argument.ends_with("paletteuse"))
        );
        let is_split_used = [&palette_pass, &encode_pass]
            .iter()
            .any(|arguments| arguments.iter().any(|argument| argument.contains("split")));
        assert!(
            !is_split_used,
            "split holds every frame until the palette is ready"
        );
        assert!(encode_pass.contains(&"-an".to_string()));
    }

    #[test]
    fn gif_size_is_capped_unless_a_lower_resolution_is_chosen() {
        let default_filter = gif_frames_filter(BALANCED);
        assert!(default_filter.contains("min(ih,480)"));

        let higher_limit = VideoOptions {
            max_height: Some(1080),
            ..BALANCED
        };
        assert!(gif_frames_filter(higher_limit).contains("min(ih,480)"));

        let lower_limit = VideoOptions {
            max_height: Some(240),
            ..BALANCED
        };
        assert!(gif_frames_filter(lower_limit).contains("min(ih,240)"));
    }
}
