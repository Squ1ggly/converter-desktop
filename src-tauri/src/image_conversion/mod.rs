mod decode;
mod encode;
mod geometry;

use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::error::{ConversionError, ConversionResult};
use crate::ffmpeg::Ffmpeg;
use crate::formats::ImageFormat;
use crate::options::ImageOptions;
use encode::EncodeJob;

pub struct ImageConversion<'a> {
    pub input_path: &'a Path,
    pub output_path: &'a Path,
    pub format: ImageFormat,
    pub options: ImageOptions,
    pub ffmpeg: &'a Ffmpeg,
    pub scratch_root: &'a Path,
    pub cancellation: &'a CancellationToken,
}

// Read → resize → write. Every source is decoded into one in-memory image so
// any input format can become any output format.
pub async fn convert_image(conversion: ImageConversion<'_>) -> ConversionResult<()> {
    let scratch_directory = tempfile::tempdir_in(conversion.scratch_root)?;

    let source = decode::load_source_image(
        conversion.input_path,
        conversion.options.resize,
        conversion.ffmpeg,
        scratch_directory.path(),
        conversion.cancellation,
    )
    .await?;
    stop_if_cancelled(conversion.cancellation)?;

    let resize = conversion.options.resize;
    let sized_image = if source.is_already_sized {
        source.image
    } else {
        run_blocking(move || Ok(geometry::resize_image(source.image, resize))).await?
    };
    stop_if_cancelled(conversion.cancellation)?;

    let encode_job = EncodeJob {
        output_path: conversion.output_path,
        format: conversion.format,
        quality: conversion.options.quality,
        ffmpeg: conversion.ffmpeg,
        scratch_directory: scratch_directory.path(),
        cancellation: conversion.cancellation,
    };
    encode::write_image(sized_image, &encode_job).await
}

fn stop_if_cancelled(cancellation: &CancellationToken) -> ConversionResult<()> {
    if cancellation.is_cancelled() {
        return Err(ConversionError::Cancelled);
    }
    Ok(())
}

// Image decoding, resizing and encoding are CPU-heavy; run them on Tokio's
// blocking pool so they don't stall the async runtime the UI events use.
pub(crate) async fn run_blocking<T: Send + 'static>(
    work: impl FnOnce() -> ConversionResult<T> + Send + 'static,
) -> ConversionResult<T> {
    let join_result = tokio::task::spawn_blocking(work).await;

    match join_result {
        Ok(work_result) => work_result,
        Err(join_error) => Err(std::io::Error::other(join_error.to_string()).into()),
    }
}
