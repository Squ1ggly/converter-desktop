use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings};
use image::{DynamicImage, ImageDecoder, ImageReader};
use resvg::{tiny_skia, usvg};
use tokio_util::sync::CancellationToken;

use super::geometry::target_dimensions;
use super::run_blocking;
use crate::error::{ConversionError, ConversionResult};
use crate::ffmpeg::Ffmpeg;
use crate::formats::lowercase_extension;
use crate::options::ResizeOption;

// PDF pages are measured in points (72 per inch); rendering at 150 DPI gives
// readable text at the page's "original size".
const PDF_RENDER_DPI: f32 = 150.0;
const PDF_POINTS_PER_INCH: f32 = 72.0;
// hayro renders into a buffer addressed with 16-bit coordinates.
const MAX_RENDER_LENGTH: u32 = u16::MAX as u32;

pub struct SourceImage {
    pub image: DynamicImage,
    // Vector sources (SVG, PDF) are rendered straight at the requested size so
    // they stay sharp; raster sources still need resizing afterwards.
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
        "pdf" => vector_source(run_blocking(move || render_pdf_first_page(&owned_input_path, resize)).await?),
        // The image crate can't decode these; the bundled FFmpeg can.
        "heic" | "heif" | "avif" => raster_source(decode_with_ffmpeg(input_path, ffmpeg, scratch_directory, cancellation).await?),
        _ => raster_source(decode_raster(input_path, ffmpeg, scratch_directory, cancellation).await?),
    };
    Ok(source_image)
}

fn vector_source(image: DynamicImage) -> SourceImage {
    SourceImage { image, is_already_sized: true }
}

fn raster_source(image: DynamicImage) -> SourceImage {
    SourceImage { image, is_already_sized: false }
}

// Files are sometimes misnamed (an iPhone HEIC saved as .jpg, say), so if the
// image crate can't read a file, give FFmpeg a try before reporting failure.
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

    let ffmpeg_result = decode_with_ffmpeg(input_path, ffmpeg, scratch_directory, cancellation).await;
    ffmpeg_result.map_err(|_| ConversionError::UnreadableImage(image_crate_error.to_string()))
}

// Applies the EXIF orientation so phone photos aren't saved sideways.
fn decode_with_image_crate(input_path: &Path) -> ConversionResult<DynamicImage> {
    let reader = ImageReader::open(input_path)?.with_guessed_format()?;
    let mut decoder = reader.into_decoder()?;
    let orientation = decoder.orientation()?;

    let mut image = DynamicImage::from_decoder(decoder)?;
    image.apply_orientation(orientation);
    Ok(image)
}

async fn decode_with_ffmpeg(
    input_path: &Path,
    ffmpeg: &Ffmpeg,
    scratch_directory: &Path,
    cancellation: &CancellationToken,
) -> ConversionResult<DynamicImage> {
    let decoded_path = scratch_directory.join("decoded.png");
    let arguments: Vec<OsString> = vec![
        "-i".into(),
        input_path.into(),
        "-frames:v".into(),
        "1".into(),
        "-pix_fmt".into(),
        "rgba".into(),
        decoded_path.clone().into(),
    ];
    ffmpeg.run(arguments, cancellation, |_| {}).await?;

    let decoded_image = run_blocking(move || decode_with_image_crate(&decoded_path)).await?;
    Ok(decoded_image)
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
        intrinsic_size.width().ceil() as u32,
        intrinsic_size.height().ceil() as u32,
        resize,
    );
    let mut pixmap = tiny_skia::Pixmap::new(target.width, target.height)
        .ok_or_else(|| ConversionError::UnreadableImage("The SVG has no visible size".to_string()))?;

    let scale_x = target.width as f32 / intrinsic_size.width();
    let scale_y = target.height as f32 / intrinsic_size.height();
    resvg::render(&tree, tiny_skia::Transform::from_scale(scale_x, scale_y), &mut pixmap.as_mut());

    let png_bytes = pixmap
        .encode_png()
        .map_err(|error| ConversionError::UnreadableImage(error.to_string()))?;
    let rendered_image = image::load_from_memory(&png_bytes)?;
    Ok(rendered_image)
}

// Loading system fonts takes a while, so do it once and share it between
// every SVG converted while the app is open.
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
fn render_pdf_first_page(input_path: &Path, resize: ResizeOption) -> ConversionResult<DynamicImage> {
    let pdf_data = std::fs::read(input_path)?;
    let pdf = Pdf::new(pdf_data).map_err(|error| ConversionError::UnreadableImage(format!("{error:?}")))?;
    let first_page = pdf
        .pages()
        .first()
        .ok_or_else(|| ConversionError::UnreadableImage("The PDF has no pages".to_string()))?;

    let (page_width_points, page_height_points) = first_page.render_dimensions();
    let original_scale = PDF_RENDER_DPI / PDF_POINTS_PER_INCH;
    let original_width = (page_width_points * original_scale).round() as u32;
    let original_height = (page_height_points * original_scale).round() as u32;

    let target = target_dimensions(original_width, original_height, resize);
    let longest_side = target.width.max(target.height);
    let size_limit_scale = (MAX_RENDER_LENGTH as f32 / longest_side as f32).min(1.0);
    let render_scale = target.width as f32 / page_width_points * size_limit_scale;

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

    let png_bytes = pixmap
        .into_png()
        .map_err(|error| ConversionError::UnreadableImage(error.to_string()))?;
    let rendered_image = image::load_from_memory(&png_bytes)?;
    Ok(rendered_image)
}
