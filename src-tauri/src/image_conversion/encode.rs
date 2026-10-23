use std::ffi::OsString;
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};

use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ImageFormat as RasterFormat};
use pdf_writer::{Content, Filter, Finish, Name, Pdf, Rect, Ref};
use tokio_util::sync::CancellationToken;

use super::geometry::{flatten_onto_white, shrink_to_fit};
use super::run_blocking;
use crate::error::{ConversionError, ConversionResult};
use crate::ffmpeg::Ffmpeg;
use crate::formats::ImageFormat;

// Tracing cost grows with pixel count and photos rarely trace well anyway;
// larger images are shrunk to this before vectorising.
const MAX_SVG_TRACE_LENGTH: u32 = 2000;
// AVIF's CRF runs 0 (lossless) to 63 (worst). Quality 100 maps to CRF 8 and
// quality 1 to CRF 62, keeping the slider's useful range spread out.
const AVIF_BEST_CRF: f32 = 8.0;
const AVIF_WORST_CRF: f32 = 62.0;
const AVIF_ENCODER_SPEED: &str = "6";

pub struct EncodeJob<'a> {
    pub output_path: &'a Path,
    pub format: ImageFormat,
    pub quality: u8,
    pub ffmpeg: &'a Ffmpeg,
    pub scratch_directory: &'a Path,
    pub cancellation: &'a CancellationToken,
}

pub async fn write_image(image: DynamicImage, job: &EncodeJob<'_>) -> ConversionResult<()> {
    let output_path = job.output_path.to_path_buf();
    let quality = job.quality.clamp(1, 100);

    match job.format {
        ImageFormat::Jpg => run_blocking(move || write_jpeg(&image, quality, &output_path)).await,
        ImageFormat::Png => {
            run_blocking(move || write_raster(&image, RasterFormat::Png, &output_path)).await
        }
        ImageFormat::Gif => {
            run_blocking(move || {
                write_raster(&image.to_rgba8().into(), RasterFormat::Gif, &output_path)
            })
            .await
        }
        ImageFormat::Tiff => {
            run_blocking(move || {
                write_raster(&image.to_rgba8().into(), RasterFormat::Tiff, &output_path)
            })
            .await
        }
        ImageFormat::Bmp => {
            run_blocking(move || {
                write_raster(
                    &flatten_onto_white(&image).into(),
                    RasterFormat::Bmp,
                    &output_path,
                )
            })
            .await
        }
        ImageFormat::Pdf => run_blocking(move || write_pdf(&image, quality, &output_path)).await,
        ImageFormat::Svg => run_blocking(move || write_svg(image, &output_path)).await,
        ImageFormat::Webp => write_with_ffmpeg(image, webp_arguments(quality), job).await,
        ImageFormat::Avif => {
            let arguments = avif_arguments(quality, image.color().has_alpha());
            write_with_ffmpeg(image, arguments, job).await
        }
        ImageFormat::Heic => write_heic(image, quality, job).await,
    }
}

fn write_raster(
    image: &DynamicImage,
    format: RasterFormat,
    output_path: &Path,
) -> ConversionResult<()> {
    let mut writer = BufWriter::new(File::create(output_path)?);
    image.write_to(&mut writer, format)?;
    Ok(())
}

fn write_jpeg(image: &DynamicImage, quality: u8, output_path: &Path) -> ConversionResult<()> {
    let flattened_image = flatten_onto_white(image);
    let writer = BufWriter::new(File::create(output_path)?);
    JpegEncoder::new_with_quality(writer, quality).encode_image(&flattened_image)?;
    Ok(())
}

// One page per image, sized so 1 pixel = 1 point (72 DPI), matching what
// image editors produce. The image is embedded as a JPEG at `quality`.
fn write_pdf(image: &DynamicImage, quality: u8, output_path: &Path) -> ConversionResult<()> {
    let flattened_image = flatten_onto_white(image);
    let (width, height) = flattened_image.dimensions();

    let mut jpeg_bytes = Vec::new();
    JpegEncoder::new_with_quality(&mut jpeg_bytes, quality).encode_image(&flattened_image)?;

    let pdf_bytes = build_single_image_pdf(&jpeg_bytes, width, height);
    std::fs::write(output_path, pdf_bytes)?;
    Ok(())
}

fn build_single_image_pdf(jpeg_bytes: &[u8], width: u32, height: u32) -> Vec<u8> {
    let catalog_id = Ref::new(1);
    let page_tree_id = Ref::new(2);
    let page_id = Ref::new(3);
    let image_id = Ref::new(4);
    let content_id = Ref::new(5);
    let image_name = Name(b"Im1");
    let page_width = width as f32;
    let page_height = height as f32;

    let mut pdf = Pdf::new();
    pdf.catalog(catalog_id).pages(page_tree_id);
    pdf.pages(page_tree_id).kids([page_id]).count(1);

    let mut page = pdf.page(page_id);
    page.media_box(Rect::new(0.0, 0.0, page_width, page_height));
    page.parent(page_tree_id);
    page.contents(content_id);
    page.resources().x_objects().pair(image_name, image_id);
    page.finish();

    let mut image_object = pdf.image_xobject(image_id, jpeg_bytes);
    image_object.filter(Filter::DctDecode);
    image_object.width(width as i32);
    image_object.height(height as i32);
    image_object.color_space().device_rgb();
    image_object.bits_per_component(8);
    image_object.finish();

    let mut content = Content::new();
    content.save_state();
    content.transform([page_width, 0.0, 0.0, page_height, 0.0, 0.0]);
    content.x_object(image_name);
    content.restore_state();
    pdf.stream(content_id, &content.finish());

    let pdf_bytes = pdf.finish();
    pdf_bytes
}

fn write_svg(image: DynamicImage, output_path: &Path) -> ConversionResult<()> {
    let traceable_image = shrink_to_fit(image, MAX_SVG_TRACE_LENGTH).to_rgba8();
    let (width, height) = traceable_image.dimensions();

    let color_image = vtracer::ColorImage {
        pixels: traceable_image.into_raw(),
        width: width as usize,
        height: height as usize,
    };
    let svg =
        vtracer::convert(color_image, vtracer::Config::default()).map_err(ConversionError::Svg)?;

    std::fs::write(output_path, svg.to_string())?;
    Ok(())
}

fn webp_arguments(quality: u8) -> Vec<OsString> {
    let arguments = vec![
        "-c:v".into(),
        "libwebp".into(),
        "-quality".into(),
        quality.to_string().into(),
    ];
    arguments
}

fn avif_arguments(quality: u8, has_transparency: bool) -> Vec<OsString> {
    let quality_fraction = (quality as f32 - 1.0) / 99.0;
    let crf = (AVIF_WORST_CRF - quality_fraction * (AVIF_WORST_CRF - AVIF_BEST_CRF)).round() as u32;

    // AV1 can't carry alpha in the colour image, so AVIF stores it as a second
    // greyscale image. This splits the alpha channel out into that stream.
    let mut arguments: Vec<OsString> = if has_transparency {
        vec![
            "-filter_complex".into(),
            "[0:v]format=yuva444p,split[color][alpha_source];[alpha_source]alphaextract[alpha]"
                .into(),
            "-map".into(),
            "[color]".into(),
            "-map".into(),
            "[alpha]".into(),
        ]
    } else {
        vec!["-pix_fmt".into(), "yuv420p".into()]
    };

    arguments.extend([
        "-c:v".into(),
        "libaom-av1".into(),
        "-still-picture".into(),
        "1".into(),
        "-crf".into(),
        crf.to_string().into(),
        "-cpu-used".into(),
        AVIF_ENCODER_SPEED.into(),
    ]);
    arguments
}

// Hands a lossless PNG of the processed image to FFmpeg for the formats our
// Rust image libraries can't encode well (lossy WebP, AVIF).
async fn write_with_ffmpeg(
    image: DynamicImage,
    encoder_arguments: Vec<OsString>,
    job: &EncodeJob<'_>,
) -> ConversionResult<()> {
    let intermediate_path = write_intermediate_png(image, job.scratch_directory).await?;

    let mut arguments: Vec<OsString> = vec!["-i".into(), intermediate_path.into()];
    arguments.extend(encoder_arguments);
    arguments.extend(["-frames:v".into(), "1".into(), job.output_path.into()]);

    job.ffmpeg.run(arguments, job.cancellation, |_| {}).await
}

async fn write_intermediate_png(
    image: DynamicImage,
    scratch_directory: &Path,
) -> ConversionResult<PathBuf> {
    let intermediate_path = scratch_directory.join("intermediate.png");
    let path_for_writer = intermediate_path.clone();
    run_blocking(move || write_raster(&image, RasterFormat::Png, &path_for_writer)).await?;
    Ok(intermediate_path)
}

// macOS ships a HEIC encoder in ImageIO; `sips` is its command-line front end
// and is present on every Mac.
async fn write_heic(image: DynamicImage, quality: u8, job: &EncodeJob<'_>) -> ConversionResult<()> {
    let intermediate_path = write_intermediate_png(image, job.scratch_directory).await?;

    let mut command = tokio::process::Command::new("/usr/bin/sips");
    command
        .args([
            "-s",
            "format",
            "heic",
            "-s",
            "formatOptions",
            &quality.to_string(),
        ])
        .arg(&intermediate_path)
        .arg("--out")
        .arg(job.output_path)
        .kill_on_drop(true);

    let output = tokio::select! {
        output = command.output() => output?,
        _ = job.cancellation.cancelled() => return Err(ConversionError::Cancelled),
    };

    if output.status.success() {
        return Ok(());
    }

    let error_message = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(ConversionError::Heic(error_message))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn crf_argument(arguments: &[OsString]) -> String {
        let crf_flag_index = arguments
            .iter()
            .position(|argument| argument == "-crf")
            .expect("has -crf");
        arguments[crf_flag_index + 1].to_string_lossy().into_owned()
    }

    #[test]
    fn avif_quality_maps_onto_the_crf_range() {
        assert_eq!(crf_argument(&avif_arguments(100, false)), "8");
        assert_eq!(crf_argument(&avif_arguments(1, false)), "62");
    }

    #[test]
    fn avif_keeps_transparency_as_a_second_stream() {
        let arguments = avif_arguments(80, true);
        assert!(arguments.iter().any(|argument| argument == "[alpha]"));
    }

    #[test]
    fn pdf_output_is_a_well_formed_document() {
        let pdf_bytes = build_single_image_pdf(b"not-really-a-jpeg", 40, 20);
        assert!(pdf_bytes.starts_with(b"%PDF-"));
        assert!(
            pdf_bytes
                .windows(b"/MediaBox [0 0 40 20]".len())
                .any(|window| window == b"/MediaBox [0 0 40 20]")
        );
    }
}
