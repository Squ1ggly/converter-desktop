use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::{LoadPdfError, Pdf};
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::vello_cpu::peniko::ImageAlphaType;
use hayro::{PixmapSettings, RenderCache, RenderSettings};
use image::{DynamicImage, ImageDecoder, ImageReader, Limits, RgbaImage};
use resvg::{tiny_skia, usvg};
use tokio_util::sync::CancellationToken;

use super::color::convert_to_srgb;
use super::geometry::{
    Dimensions, MAX_IMAGE_PIXELS, ensure_source_fits, ensure_target_fits, target_dimensions,
    whole_pixels,
};
use super::run_blocking;
use crate::error::{ConversionError, ConversionResult};
use crate::ffmpeg::Ffmpeg;
use crate::formats::lowercase_extension;
use crate::options::ResizeOption;

// "Original size" for a PDF page.
const PDF_RENDER_DPI: f32 = 150.0;
const PDF_POINTS_PER_INCH: f32 = 72.0;
// hayro::render sizes its pixmap in u16, so a longer side would be cut off.
const MAX_RENDER_LENGTH: u32 = u16::MAX as u32;
// 16-bit RGBA, the largest layout a decoder allocates for an image in budget.
const MAX_DECODE_BYTES_PER_PIXEL: u64 = 8;

pub struct SourceImage {
    pub image: DynamicImage,
    // SVG and PDF render at the target size, so they skip the resize step.
    pub is_already_sized: bool,
}

pub async fn load_source_image(
    input_path: &Path,
    resize: ResizeOption,
    ffmpeg: &Ffmpeg,
    scratch_directory: &Path,
    cancellation: &CancellationToken,
) -> ConversionResult<SourceImage> {
    let extension = lowercase_extension(input_path);
    let owned_input_path = input_path.to_path_buf();

    let source_image = match extension.as_str() {
        "svg" => vector_source(run_blocking(move || render_svg(&owned_input_path, resize)).await?),
        "pdf" => vector_source(
            run_blocking(move || render_pdf_first_page(&owned_input_path, resize)).await?,
        ),
        // The image crate can't decode these; the bundled FFmpeg can.
        "heic" | "heif" | "avif" => raster_source(
            decode_with_ffmpeg(input_path, ffmpeg, scratch_directory, cancellation).await?,
        ),
        _ => {
            raster_source(decode_raster(input_path, ffmpeg, scratch_directory, cancellation).await?)
        }
    };
    Ok(source_image)
}

fn vector_source(image: DynamicImage) -> SourceImage {
    SourceImage {
        image,
        is_already_sized: true,
    }
}

fn raster_source(image: DynamicImage) -> SourceImage {
    SourceImage {
        image,
        is_already_sized: false,
    }
}

// Falls back to FFmpeg for misnamed files, e.g. a HEIC saved as .jpg.
async fn decode_raster(
    input_path: &Path,
    ffmpeg: &Ffmpeg,
    scratch_directory: &Path,
    cancellation: &CancellationToken,
) -> ConversionResult<DynamicImage> {
    let owned_input_path = input_path.to_path_buf();
    let image_crate_result = run_blocking(move || decode_with_image_crate(&owned_input_path)).await;

    let Err(image_crate_error) = image_crate_result else {
        return image_crate_result;
    };
    if matches!(image_crate_error, ConversionError::ImageTooLarge) {
        return Err(image_crate_error);
    }

    // FFmpeg's error would be about a format the file was never meant to be,
    // so the image crate's is the one reported.
    let decoded_image = decode_with_ffmpeg(input_path, ffmpeg, scratch_directory, cancellation)
        .await
        .map_err(|_| ConversionError::UnreadableImage(image_crate_error.to_string()))?;
    Ok(decoded_image)
}

// Applies the EXIF orientation so phone photos aren't saved sideways, and
// converts colours to sRGB.
fn decode_with_image_crate(input_path: &Path) -> ConversionResult<DynamicImage> {
    let mut reader = ImageReader::open(input_path)?.with_guessed_format()?;
    reader.limits(decode_limits());
    let mut decoder = reader.into_decoder().map_err(describe_decode_error)?;
    let (width, height) = decoder.dimensions();
    ensure_source_fits(Dimensions { width, height })?;
    let orientation = decoder.orientation()?;
    let icc_profile = decoder.icc_profile()?;

    let decoded_image = DynamicImage::from_decoder(decoder).map_err(describe_decode_error)?;
    let mut image = convert_to_srgb(decoded_image, icc_profile.as_deref());
    image.apply_orientation(orientation);
    Ok(image)
}

fn describe_decode_error(error: image::ImageError) -> ConversionError {
    if matches!(error, image::ImageError::Limits(_)) {
        return ConversionError::ImageTooLarge;
    }
    ConversionError::Image(error)
}

// The image crate's own default (512 MiB) would refuse 16-bit images that fit
// the pixel budget.
fn decode_limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_IMAGE_PIXELS * MAX_DECODE_BYTES_PER_PIXEL);
    limits
}

async fn decode_with_ffmpeg(
    input_path: &Path,
    ffmpeg: &Ffmpeg,
    scratch_directory: &Path,
    cancellation: &CancellationToken,
) -> ConversionResult<DynamicImage> {
    let readable_input_path = path_ffmpeg_can_read(input_path, scratch_directory)?;
    let decoded_path = scratch_directory.join("decoded.png");
    let arguments: Vec<OsString> = vec![
        "-i".into(),
        readable_input_path.into(),
        "-frames:v".into(),
        "1".into(),
        "-pix_fmt".into(),
        "rgba".into(),
        "-f".into(),
        "image2".into(),
        "-update".into(),
        "1".into(),
        decoded_path.clone().into(),
    ];
    ffmpeg.run(arguments, cancellation, |_| {}).await?;

    let decoded_image = run_blocking(move || decode_with_image_crate(&decoded_path)).await?;
    Ok(decoded_image)
}

// FFmpeg reads "%d" in a path with an image extension as an image sequence
// and picks its decoder from the extension, so a misnamed "Week 3%done.jpg"
// fails. A link without an extension makes FFmpeg go by the content.
fn path_ffmpeg_can_read(input_path: &Path, scratch_directory: &Path) -> ConversionResult<PathBuf> {
    let has_percent_sign = input_path.as_os_str().to_string_lossy().contains('%');
    if !has_percent_sign {
        return Ok(input_path.to_path_buf());
    }

    let linked_path = scratch_directory.join("source");
    if std::fs::hard_link(input_path, &linked_path).is_err() {
        std::fs::copy(input_path, &linked_path)?;
    }
    Ok(linked_path)
}

fn render_svg(input_path: &Path, resize: ResizeOption) -> ConversionResult<DynamicImage> {
    let svg_data = std::fs::read(input_path)?;
    let options = usvg::Options {
        resources_dir: input_path.parent().map(PathBuf::from),
        fontdb: shared_font_database(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(&svg_data, &options)
        .map_err(|error| ConversionError::UnreadableImage(error.to_string()))?;

    let intrinsic_size = tree.size();
    let target = target_dimensions(
        whole_pixels(intrinsic_size.width()),
        whole_pixels(intrinsic_size.height()),
        resize,
    );
    ensure_target_fits(target)?;
    let mut pixmap = tiny_skia::Pixmap::new(target.width, target.height).ok_or_else(|| {
        ConversionError::UnreadableImage("The SVG has no visible size".to_string())
    })?;

    let scale_x = target.width as f32 / intrinsic_size.width();
    let scale_y = target.height as f32 / intrinsic_size.height();
    resvg::render(
        &tree,
        tiny_skia::Transform::from_scale(scale_x, scale_y),
        &mut pixmap.as_mut(),
    );

    // tiny-skia stores premultiplied alpha; image expects straight alpha.
    let rendered_image =
        image_from_rgba_pixels(target.width, target.height, pixmap.take_demultiplied())?;
    Ok(rendered_image)
}

fn image_from_rgba_pixels(
    width: u32,
    height: u32,
    rgba_pixels: Vec<u8>,
) -> ConversionResult<DynamicImage> {
    let rgba_image = RgbaImage::from_raw(width, height, rgba_pixels).ok_or_else(|| {
        ConversionError::UnreadableImage("The rendered image has the wrong size".to_string())
    })?;
    let rendered_image = DynamicImage::ImageRgba8(rgba_image);
    Ok(rendered_image)
}

fn describe_pdf_error(error: LoadPdfError) -> ConversionError {
    let reason = match error {
        LoadPdfError::Decryption(_) => "the PDF is password-protected",
        LoadPdfError::Invalid => "the PDF is damaged",
    };
    ConversionError::UnreadableImage(reason.to_string())
}

// Loading system fonts is slow, so it happens once.
fn shared_font_database() -> Arc<usvg::fontdb::Database> {
    static FONT_DATABASE: OnceLock<Arc<usvg::fontdb::Database>> = OnceLock::new();

    let font_database = FONT_DATABASE.get_or_init(|| {
        let mut database = usvg::fontdb::Database::new();
        database.load_system_fonts();
        Arc::new(database)
    });
    font_database.clone()
}

// Multi-page PDFs convert their first page, the one people expect to see.
fn render_pdf_first_page(
    input_path: &Path,
    resize: ResizeOption,
) -> ConversionResult<DynamicImage> {
    let pdf_data = std::fs::read(input_path)?;
    let pdf = Pdf::new(pdf_data).map_err(describe_pdf_error)?;
    let first_page = pdf
        .pages()
        .first()
        .ok_or_else(|| ConversionError::UnreadableImage("The PDF has no pages".to_string()))?;

    let (page_width_points, page_height_points) = first_page.render_dimensions();
    let original_scale = PDF_RENDER_DPI / PDF_POINTS_PER_INCH;
    let original_width = whole_pixels((page_width_points * original_scale).round());
    let original_height = whole_pixels((page_height_points * original_scale).round());

    let target = target_dimensions(original_width, original_height, resize);
    let longest_side = target.width.max(target.height);
    let size_limit_scale = (MAX_RENDER_LENGTH as f32 / longest_side as f32).min(1.0);
    let render_scale = target.width as f32 / page_width_points * size_limit_scale;
    let render_dimensions = Dimensions {
        width: whole_pixels(page_width_points * render_scale),
        height: whole_pixels(page_height_points * render_scale),
    };
    ensure_target_fits(render_dimensions)?;

    let pixmap_settings = PixmapSettings {
        x_scale: render_scale,
        y_scale: render_scale,
        bg_color: WHITE,
    };
    let pixmap = hayro::render(
        first_page,
        &RenderCache::new(),
        &InterpreterSettings::default(),
        &RenderSettings::default(),
        &pixmap_settings,
    );

    let width = u32::from(pixmap.width());
    let height = u32::from(pixmap.height());
    let rendered_image =
        image_from_rgba_pixels(width, height, pixmap.take_rgba8(ImageAlphaType::Alpha))?;
    Ok(rendered_image)
}
