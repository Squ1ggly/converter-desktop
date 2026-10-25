// End-to-end checks that run real conversions through the bundled FFmpeg:
// every kind of input becomes every output format, and each result is read
// back to prove it's a valid file (not just that no error was returned).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};
use tokio_util::sync::CancellationToken;

use crate::error::ConversionError;
use crate::ffmpeg::Ffmpeg;
use crate::formats::{ImageFormat, VideoFormat};
use crate::image_conversion::{ImageConversion, convert_image};
use crate::options::{ImageOptions, ResizeOption, VideoOptions, VideoQuality};
use crate::video_conversion::{VideoConversion, convert_video};

const ALL_IMAGE_OUTPUTS: [ImageFormat; 10] = [
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

const ALL_VIDEO_OUTPUTS: [VideoFormat; 8] = [
    VideoFormat::Mp4,
    VideoFormat::Mov,
    VideoFormat::Mkv,
    VideoFormat::Webm,
    VideoFormat::M4v,
    VideoFormat::Avi,
    VideoFormat::Flv,
    VideoFormat::Gif,
];

// (folder, file stem) pairs. FFmpeg's image2 muxer and demuxer read a path
// with exactly one "%d" as a numbered sequence, so each case has at most one.
const AWKWARD_NAMES: [(&str, &str); 3] = [
    ("plain", "Week 3%done 照片 é"),
    ("Week 3%done 照片", "photo é"),
    ("100% sure", "50% off"),
];

const SAMPLE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="64" height="48">
  <rect width="64" height="48" fill="#4f46e5"/><circle cx="32" cy="24" r="16" fill="white"/>
</svg>"##;

fn image_outputs_for_this_platform() -> Vec<ImageFormat> {
    let outputs = ALL_IMAGE_OUTPUTS
        .into_iter()
        .filter(|format| *format != ImageFormat::Heic || cfg!(target_os = "macos"))
        .collect();
    outputs
}

// A half-transparent circle on a transparent background, with odd dimensions
// to catch encoders that need even sizes.
fn write_transparent_png(path: &Path) {
    let image = RgbaImage::from_fn(121, 81, |x, y| {
        let distance_x = x as f32 - 60.0;
        let distance_y = y as f32 - 40.0;
        let is_inside_circle = distance_x * distance_x + distance_y * distance_y < 35.0 * 35.0;
        if is_inside_circle {
            Rgba([220, 40, 40, 160])
        } else {
            Rgba([0, 0, 0, 0])
        }
    });
    image.save(path).expect("write sample png");
}

async fn run_ffmpeg(ffmpeg: &Ffmpeg, arguments: &[&str]) {
    let arguments: Vec<OsString> = arguments.iter().map(OsString::from).collect();
    ffmpeg
        .run(arguments, &CancellationToken::new(), |_| {})
        .await
        .expect("ffmpeg helper command");
}

// Decoding the whole file with FFmpeg is the strictest "is this valid" check
// available for raster images and videos.
async fn assert_decodes(ffmpeg: &Ffmpeg, path: &Path) {
    let arguments: Vec<OsString> = vec![
        "-v".into(),
        "error".into(),
        "-i".into(),
        path.into(),
        "-f".into(),
        "null".into(),
        "-".into(),
    ];
    let decode_result = ffmpeg
        .run(arguments, &CancellationToken::new(), |_| {})
        .await;
    assert!(
        decode_result.is_ok(),
        "{} did not decode: {decode_result:?}",
        path.display()
    );
}

async fn assert_valid_output(ffmpeg: &Ffmpeg, path: &Path) {
    let bytes = std::fs::read(path).expect("output exists");
    assert!(!bytes.is_empty(), "{} is empty", path.display());

    let extension = path.extension().unwrap().to_string_lossy().into_owned();
    match extension.as_str() {
        "pdf" => assert!(
            bytes.starts_with(b"%PDF-"),
            "{} is not a PDF",
            path.display()
        ),
        "svg" => assert!(
            String::from_utf8_lossy(&bytes).contains("<svg"),
            "{} is not an SVG",
            path.display()
        ),
        _ => assert_decodes(ffmpeg, path).await,
    }
}

async fn convert_image_to(
    input: &Path,
    format: ImageFormat,
    resize: ResizeOption,
    output_directory: &Path,
) -> Result<PathBuf, ConversionError> {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let input_stem = input.file_stem().unwrap().to_string_lossy();
    let output_path = output_directory.join(format!(
        "{input_stem}-{}.{}",
        input.extension().unwrap().to_string_lossy(),
        format.extension()
    ));

    let conversion = ImageConversion {
        input_path: input,
        output_path: &output_path,
        format,
        options: ImageOptions {
            quality: 80,
            resize,
        },
        ffmpeg: &ffmpeg,
        scratch_root: output_directory,
        cancellation: &CancellationToken::new(),
    };
    convert_image(conversion).await?;
    Ok(output_path)
}

async fn prepare_image_inputs(directory: &Path) -> Vec<PathBuf> {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let png_path = directory.join("sample.png");
    write_transparent_png(&png_path);

    let svg_path = directory.join("vector.svg");
    std::fs::write(&svg_path, SAMPLE_SVG).unwrap();

    let jpeg_path = directory.join("photo.jpg");
    let avif_path = directory.join("modern.avif");
    let webp_path = directory.join("web.webp");
    let png_text = png_path.to_string_lossy().into_owned();
    run_ffmpeg(&ffmpeg, &["-i", &png_text, &jpeg_path.to_string_lossy()]).await;
    run_ffmpeg(
        &ffmpeg,
        &[
            "-i",
            &png_text,
            "-c:v",
            "libaom-av1",
            "-still-picture",
            "1",
            &avif_path.to_string_lossy(),
        ],
    )
    .await;
    run_ffmpeg(
        &ffmpeg,
        &[
            "-i",
            &png_text,
            "-c:v",
            "libwebp",
            &webp_path.to_string_lossy(),
        ],
    )
    .await;

    // Our own PDF output doubles as a PDF input.
    let pdf_path = convert_image_to(
        &png_path,
        ImageFormat::Pdf,
        ResizeOption::Original,
        directory,
    )
    .await
    .unwrap();

    let mut inputs = vec![
        png_path, svg_path, jpeg_path, avif_path, webp_path, pdf_path,
    ];
    if cfg!(target_os = "macos") {
        let heic_path = convert_image_to(
            &inputs[0],
            ImageFormat::Heic,
            ResizeOption::Original,
            directory,
        )
        .await
        .unwrap();
        inputs.push(heic_path);
    }
    inputs
}

#[tokio::test(flavor = "multi_thread")]
async fn every_image_input_converts_to_every_image_output() {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let input_directory = tempfile::tempdir().unwrap();
    let output_directory = tempfile::tempdir().unwrap();
    let inputs = prepare_image_inputs(input_directory.path()).await;

    for input in &inputs {
        for format in image_outputs_for_this_platform() {
            let conversion_result = convert_image_to(
                input,
                format,
                ResizeOption::Original,
                output_directory.path(),
            )
            .await;
            let output_path = conversion_result
                .unwrap_or_else(|error| panic!("{} → {format:?} failed: {error}", input.display()));
            assert_valid_output(&ffmpeg, &output_path).await;
        }
    }
}

fn file_names_in(directory: &Path) -> Vec<String> {
    let mut file_names: Vec<String> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    file_names.sort();
    file_names
}

#[tokio::test(flavor = "multi_thread")]
async fn awkward_file_and_folder_names_convert_to_every_image_output() {
    for (folder_name, file_stem) in AWKWARD_NAMES {
        convert_awkward_image_to_every_output(folder_name, file_stem).await;
    }
}

async fn convert_awkward_image_to_every_output(folder_name: &str, file_stem: &str) {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let input_directory = tempfile::tempdir().unwrap();
    let output_root = tempfile::tempdir().unwrap();
    let output_directory = output_root.path().join(folder_name);
    std::fs::create_dir(&output_directory).unwrap();
    let input_path = input_directory.path().join(format!("{file_stem}.png"));
    write_transparent_png(&input_path);

    for format in image_outputs_for_this_platform() {
        let output_path = convert_image_to(
            &input_path,
            format,
            ResizeOption::Original,
            &output_directory,
        )
        .await
        .unwrap_or_else(|error| panic!("{file_stem} → {format:?} failed: {error}"));
        assert_valid_output(&ffmpeg, &output_path).await;
    }

    // Nothing else may appear, such as "Week 31one" from a muxer that read the
    // name as a pattern. macOS `sips` stores the HEIC name decomposed, so
    // names are looked up rather than compared.
    let written_file_count = file_names_in(&output_directory)
        .iter()
        .filter(|name| !name.starts_with(".tmp"))
        .count();
    assert_eq!(written_file_count, image_outputs_for_this_platform().len());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_misnamed_file_with_a_percent_sign_is_still_read() {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let directory = tempfile::tempdir().unwrap();
    let png_path = directory.path().join("source.png");
    write_transparent_png(&png_path);

    // AVIF content the image crate can't read, under a name FFmpeg's image2
    // demuxer would claim as a numbered sequence.
    let misnamed_path = directory.path().join("Week 3%done.png");
    run_ffmpeg(
        &ffmpeg,
        &[
            "-i",
            &png_path.to_string_lossy(),
            "-c:v",
            "libaom-av1",
            "-still-picture",
            "1",
            "-f",
            "avif",
            &misnamed_path.to_string_lossy(),
        ],
    )
    .await;

    let output_path = convert_image_to(
        &misnamed_path,
        ImageFormat::Png,
        ResizeOption::Original,
        directory.path(),
    )
    .await
    .expect("misnamed file converts");
    assert_eq!(image::image_dimensions(&output_path).unwrap(), (121, 81));
}

#[tokio::test(flavor = "multi_thread")]
async fn resizing_applies_to_raster_and_vector_inputs() {
    let input_directory = tempfile::tempdir().unwrap();
    let inputs = prepare_image_inputs(input_directory.path()).await;
    let fit_width = ResizeOption::Fit {
        max_width: Some(200),
        max_height: None,
    };

    for input in inputs
        .iter()
        .filter(|input| !input.to_string_lossy().ends_with(".pdf"))
    {
        let output_path =
            convert_image_to(input, ImageFormat::Png, fit_width, input_directory.path())
                .await
                .unwrap();
        let (width, _height) = image::image_dimensions(&output_path).unwrap();
        assert_eq!(width, 200, "{} was not resized", input.display());
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn every_video_output_is_playable() {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let directory = tempfile::tempdir().unwrap();
    let input_path = directory.path().join("clip.mov");

    // Odd frame size to check every encoder copes with it.
    run_ffmpeg(
        &ffmpeg,
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=1:size=321x241:rate=24",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=1",
            "-shortest",
            &input_path.to_string_lossy(),
        ],
    )
    .await;

    for format in ALL_VIDEO_OUTPUTS {
        let output_path = directory
            .path()
            .join(format!("converted.{}", format.extension()));
        let mut last_progress = 0.0_f32;
        let conversion = VideoConversion {
            input_path: &input_path,
            output_path: &output_path,
            format,
            options: VideoOptions {
                quality: VideoQuality::Small,
                max_height: Some(120),
                frame_rate: None,
            },
            ffmpeg: &ffmpeg,
            scratch_root: directory.path(),
            cancellation: &CancellationToken::new(),
        };

        let conversion_result = convert_video(conversion, |percent| last_progress = percent).await;
        assert!(
            conversion_result.is_ok(),
            "clip → {format:?} failed: {conversion_result:?}"
        );
        assert_valid_output(&ffmpeg, &output_path).await;
        assert!(last_progress > 0.0, "{format:?} reported no progress");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn awkward_file_and_folder_names_convert_to_every_video_output() {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let input_directory = tempfile::tempdir().unwrap();
    let input_path = input_directory.path().join("clip.mov");
    run_ffmpeg(
        &ffmpeg,
        &[
            "-f",
            "lavfi",
            "-i",
            "testsrc=duration=0.5:size=64x48:rate=10",
            &input_path.to_string_lossy(),
        ],
    )
    .await;

    for (folder_name, file_stem) in AWKWARD_NAMES {
        let output_root = tempfile::tempdir().unwrap();
        let output_directory = output_root.path().join(folder_name);
        std::fs::create_dir(&output_directory).unwrap();

        for format in ALL_VIDEO_OUTPUTS {
            let output_path = output_directory.join(format!("{file_stem}.{}", format.extension()));
            convert_quick_video(&ffmpeg, &input_path, &output_path, format).await;
            assert_valid_output(&ffmpeg, &output_path).await;
        }
        assert_eq!(
            file_names_in(&output_directory).len(),
            ALL_VIDEO_OUTPUTS.len()
        );
    }
}

async fn convert_quick_video(
    ffmpeg: &Ffmpeg,
    input_path: &Path,
    output_path: &Path,
    format: VideoFormat,
) {
    let conversion = VideoConversion {
        input_path,
        output_path,
        format,
        options: VideoOptions {
            quality: VideoQuality::Small,
            max_height: None,
            frame_rate: None,
        },
        ffmpeg,
        scratch_root: output_path.parent().unwrap(),
        cancellation: &CancellationToken::new(),
    };
    let conversion_result = convert_video(conversion, |_| {}).await;
    assert!(
        conversion_result.is_ok(),
        "{} failed: {conversion_result:?}",
        output_path.display()
    );
}

// Reads "WxH" and fps from FFmpeg's description of the first video stream.
fn describe_video(ffmpeg: &Ffmpeg, path: &Path) -> (u32, u32, f32) {
    let output = std::process::Command::new(ffmpeg.binary_path())
        .arg("-i")
        .arg(path)
        .output()
        .expect("run ffmpeg");
    let log = String::from_utf8_lossy(&output.stderr);
    let video_line = log
        .lines()
        .find(|line| line.contains("Video:"))
        .expect("video stream");

    let size = video_line
        .split(", ")
        .find_map(|part| {
            let (width, height) = part.split_whitespace().next()?.split_once('x')?;
            Some((width.parse().ok()?, height.parse().ok()?))
        })
        .expect("frame size");
    let frame_rate = video_line
        .split(", ")
        .find_map(|part| part.strip_suffix(" fps")?.parse().ok())
        .expect("frame rate");

    let description = (size.0, size.1, frame_rate);
    description
}

async fn convert_test_clip(source: &str, options: VideoOptions) -> (u32, u32, f32) {
    convert_test_clip_to(source, VideoFormat::Mp4, options).await
}

async fn convert_test_clip_to(
    source: &str,
    format: VideoFormat,
    options: VideoOptions,
) -> (u32, u32, f32) {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let directory = tempfile::tempdir().unwrap();
    let input_path = directory.path().join("source.mp4");
    let output_path = directory
        .path()
        .join(format!("converted.{}", format.extension()));
    run_ffmpeg(
        &ffmpeg,
        &["-f", "lavfi", "-i", source, &input_path.to_string_lossy()],
    )
    .await;

    let conversion = VideoConversion {
        input_path: &input_path,
        output_path: &output_path,
        format,
        options,
        ffmpeg: &ffmpeg,
        scratch_root: directory.path(),
        cancellation: &CancellationToken::new(),
    };
    convert_video(conversion, |_| {})
        .await
        .expect("convert clip");

    let description = describe_video(&ffmpeg, &output_path);
    description
}

#[tokio::test(flavor = "multi_thread")]
async fn resolution_limits_the_shorter_side_of_portrait_video() {
    let options = VideoOptions {
        quality: VideoQuality::Small,
        max_height: Some(120),
        frame_rate: Some(15),
    };
    let (width, height, frame_rate) =
        convert_test_clip("testsrc=duration=1:size=240x480:rate=30", options).await;

    assert_eq!((width, height), (120, 240));
    assert_eq!(frame_rate, 15.0);
}

#[tokio::test(flavor = "multi_thread")]
async fn limits_never_enlarge_or_speed_up_video() {
    let options = VideoOptions {
        quality: VideoQuality::Small,
        max_height: Some(1080),
        frame_rate: Some(60),
    };
    let (width, height, frame_rate) =
        convert_test_clip("testsrc=duration=1:size=480x240:rate=24", options).await;

    assert_eq!((width, height), (480, 240));
    assert_eq!(frame_rate, 24.0);
}

#[tokio::test(flavor = "multi_thread")]
async fn gif_output_is_capped_at_480_pixels_on_the_shorter_side() {
    let options = VideoOptions {
        quality: VideoQuality::Balanced,
        max_height: None,
        frame_rate: None,
    };
    let (width, height, _frame_rate) = convert_test_clip_to(
        "testsrc=duration=0.5:size=1280x720:rate=10",
        VideoFormat::Gif,
        options,
    )
    .await;

    assert_eq!((width, height), (853, 480));
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelling_stops_ffmpeg() {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let directory = tempfile::tempdir().unwrap();
    let output_path = directory.path().join("endless.mp4");
    let cancellation = CancellationToken::new();

    let cancel_soon = cancellation.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        cancel_soon.cancel();
    });

    // A 10-minute source that would take far longer than the test to encode.
    let arguments: Vec<OsString> = [
        "-f",
        "lavfi",
        "-i",
        "testsrc=duration=600:size=1280x720",
        &output_path.to_string_lossy(),
    ]
    .iter()
    .map(OsString::from)
    .collect();
    let started_at = std::time::Instant::now();
    let run_result = ffmpeg.run(arguments, &cancellation, |_| {}).await;

    assert!(matches!(run_result, Err(ConversionError::Cancelled)));
    assert!(started_at.elapsed() < std::time::Duration::from_secs(5));
    // FFmpeg has exited by now, so even on Windows nothing holds the files open.
    directory
        .close()
        .expect("remove the folder FFmpeg was writing to");
}

fn assert_too_large(conversion_result: &Result<PathBuf, ConversionError>, input: &str) {
    assert!(
        matches!(
            conversion_result,
            Err(ConversionError::ImageTooLarge | ConversionError::TargetTooLarge)
        ),
        "{input}: expected a size error, got {conversion_result:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn sizes_beyond_the_pixel_budget_fail_cleanly() {
    let directory = tempfile::tempdir().unwrap();
    let png_path = directory.path().join("small.png");
    write_transparent_png(&png_path);
    let unbounded_fit = ResizeOption::Fit {
        max_width: Some(u32::MAX),
        max_height: None,
    };
    let upscale_result =
        convert_image_to(&png_path, ImageFormat::Png, unbounded_fit, directory.path()).await;
    assert_too_large(&upscale_result, "PNG fit to u32::MAX");

    // 144 megapixels, drawn at the requested size before anything else.
    let svg_path = directory.path().join("huge.svg");
    std::fs::write(
        &svg_path,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="12000" height="12000"/>"#,
    )
    .unwrap();
    let svg_result = convert_image_to(
        &svg_path,
        ImageFormat::Png,
        ResizeOption::Original,
        directory.path(),
    )
    .await;
    assert_too_large(&svg_result, "12000 × 12000 SVG");

    // A 1210 × 810 point page is 2521 × 1688 at 150 DPI, so 1000% is 425 megapixels.
    let large_png_path = convert_image_to(
        &png_path,
        ImageFormat::Png,
        ResizeOption::Percent { percent: 1000 },
        directory.path(),
    )
    .await
    .unwrap();
    let pdf_path = convert_image_to(
        &large_png_path,
        ImageFormat::Pdf,
        ResizeOption::Original,
        directory.path(),
    )
    .await
    .unwrap();
    let pdf_result = convert_image_to(
        &pdf_path,
        ImageFormat::Png,
        ResizeOption::Percent { percent: 1000 },
        directory.path(),
    )
    .await;
    assert_too_large(&pdf_result, "PDF at 1000%");
}

// A BMP header that claims 20000 × 20000 pixels with no pixel data, so the
// size has to be rejected from the header alone.
fn write_bmp_header_only(path: &Path, width: u32, height: u32) {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"BM");
    bytes.extend_from_slice(&54_u32.to_le_bytes());
    bytes.extend_from_slice(&0_u32.to_le_bytes());
    bytes.extend_from_slice(&54_u32.to_le_bytes());
    bytes.extend_from_slice(&40_u32.to_le_bytes());
    bytes.extend_from_slice(&width.to_le_bytes());
    bytes.extend_from_slice(&height.to_le_bytes());
    bytes.extend_from_slice(&1_u16.to_le_bytes());
    bytes.extend_from_slice(&24_u16.to_le_bytes());
    bytes.extend_from_slice(&[0; 24]);
    std::fs::write(path, bytes).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn a_source_beyond_the_pixel_budget_is_rejected_before_decoding() {
    let directory = tempfile::tempdir().unwrap();
    let bmp_path = directory.path().join("huge.bmp");
    write_bmp_header_only(&bmp_path, 20_000, 20_000);

    let conversion_result = convert_image_to(
        &bmp_path,
        ImageFormat::Png,
        ResizeOption::Percent { percent: 1 },
        directory.path(),
    )
    .await;

    assert!(
        matches!(conversion_result, Err(ConversionError::ImageTooLarge)),
        "got {conversion_result:?}"
    );
}

// Within the budget but wider than JPEG, GIF and WebP allow. Each output
// either works or fails with an error, never a panic.
#[tokio::test(flavor = "multi_thread")]
async fn very_wide_images_convert_or_fail_cleanly() {
    let ffmpeg = Ffmpeg::from_downloaded_binary();
    let directory = tempfile::tempdir().unwrap();
    let wide_path = directory.path().join("wide.png");
    RgbaImage::from_pixel(70_000, 2, Rgba([10, 120, 200, 255]))
        .save(&wide_path)
        .unwrap();

    for format in image_outputs_for_this_platform() {
        let conversion_result =
            convert_image_to(&wide_path, format, ResizeOption::Original, directory.path()).await;
        match conversion_result {
            Ok(output_path) => assert_valid_output(&ffmpeg, &output_path).await,
            Err(error) => {
                let message = error.to_string();
                assert!(!message.contains("panicked"), "{format:?}: {message}");
            }
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn svg_transparency_keeps_its_colour() {
    let directory = tempfile::tempdir().unwrap();
    let svg_path = directory.path().join("half.svg");
    std::fs::write(
        &svg_path,
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8">
  <rect width="8" height="8" fill="#ff0000" fill-opacity="0.5"/>
</svg>"##,
    )
    .unwrap();

    let output_path = convert_image_to(
        &svg_path,
        ImageFormat::Png,
        ResizeOption::Original,
        directory.path(),
    )
    .await
    .unwrap();

    let pixel = image::open(&output_path)
        .unwrap()
        .to_rgba8()
        .get_pixel(4, 4)
        .0;
    assert_eq!(pixel[0], 255, "colour was premultiplied: {pixel:?}");
    assert!((127..=128).contains(&pixel[3]), "alpha: {pixel:?}");
}
