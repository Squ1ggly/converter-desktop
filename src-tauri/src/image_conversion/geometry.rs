use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Rgb, RgbImage};

use crate::options::ResizeOption;

const MIN_PERCENT: u32 = 1;
const MAX_PERCENT: u32 = 1000;

pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

// "Fit" scales up or down to fill the box, keeping proportions. A blank side
// doesn't constrain.
pub fn target_dimensions(width: u32, height: u32, resize: ResizeOption) -> Dimensions {
    let scale_factor = match resize {
        ResizeOption::Original => None,
        ResizeOption::Percent { percent } => {
            Some(percent.clamp(MIN_PERCENT, MAX_PERCENT) as f64 / 100.0)
        }
        ResizeOption::Fit {
            max_width,
            max_height,
        } => fit_scale_factor(width, height, max_width, max_height),
    };

    let Some(scale_factor) = scale_factor else {
        return Dimensions { width, height };
    };

    let scaled_dimensions = Dimensions {
        width: scale_length(width, scale_factor),
        height: scale_length(height, scale_factor),
    };
    scaled_dimensions
}

fn fit_scale_factor(
    width: u32,
    height: u32,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Option<f64> {
    let width_ratio = max_width
        .filter(|limit| *limit > 0)
        .map(|limit| limit as f64 / width as f64);
    let height_ratio = max_height
        .filter(|limit| *limit > 0)
        .map(|limit| limit as f64 / height as f64);

    let scale_factor = match (width_ratio, height_ratio) {
        (Some(width_ratio), Some(height_ratio)) => Some(width_ratio.min(height_ratio)),
        (Some(width_ratio), None) => Some(width_ratio),
        (None, Some(height_ratio)) => Some(height_ratio),
        (None, None) => None,
    };
    scale_factor
}

fn scale_length(length: u32, scale_factor: f64) -> u32 {
    let scaled_length = (length as f64 * scale_factor).round().max(1.0) as u32;
    scaled_length
}

pub fn resize_image(image: DynamicImage, resize: ResizeOption) -> DynamicImage {
    let (width, height) = image.dimensions();
    let target = target_dimensions(width, height, resize);

    let is_unchanged = target.width == width && target.height == height;
    if is_unchanged {
        return image;
    }

    let resized_image = image.resize_exact(target.width, target.height, FilterType::Lanczos3);
    resized_image
}

// Shrinks (never enlarges) so neither side exceeds `max_length`.
pub fn shrink_to_fit(image: DynamicImage, max_length: u32) -> DynamicImage {
    let (width, height) = image.dimensions();
    let fits_already = width <= max_length && height <= max_length;
    if fits_already {
        return image;
    }

    let shrunk_image = image.resize(max_length, max_length, FilterType::Lanczos3);
    shrunk_image
}

// For formats without transparency, which would otherwise show black.
pub fn flatten_onto_white(image: &DynamicImage) -> RgbImage {
    let rgba_image = image.to_rgba8();
    let (width, height) = rgba_image.dimensions();

    let flattened_image = RgbImage::from_fn(width, height, |x, y| {
        let [red, green, blue, alpha] = rgba_image.get_pixel(x, y).0;
        Rgb([
            blend_channel_onto_white(red, alpha),
            blend_channel_onto_white(green, alpha),
            blend_channel_onto_white(blue, alpha),
        ])
    });
    flattened_image
}

fn blend_channel_onto_white(channel: u8, alpha: u8) -> u8 {
    let opacity = alpha as u32;
    let blended = (channel as u32 * opacity + 255 * (255 - opacity) + 127) / 255;
    blended as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dimensions_of(width: u32, height: u32, resize: ResizeOption) -> (u32, u32) {
        let target = target_dimensions(width, height, resize);
        (target.width, target.height)
    }

    #[test]
    fn fit_keeps_proportions_inside_the_box() {
        let fit_box = ResizeOption::Fit {
            max_width: Some(800),
            max_height: Some(800),
        };
        assert_eq!(dimensions_of(4000, 3000, fit_box), (800, 600));
        assert_eq!(dimensions_of(400, 300, fit_box), (800, 600));
    }

    #[test]
    fn fit_with_one_side_blank_only_constrains_the_other() {
        let width_only = ResizeOption::Fit {
            max_width: Some(1000),
            max_height: None,
        };
        assert_eq!(dimensions_of(4000, 3000, width_only), (1000, 750));

        let nothing = ResizeOption::Fit {
            max_width: None,
            max_height: None,
        };
        assert_eq!(dimensions_of(4000, 3000, nothing), (4000, 3000));
    }

    #[test]
    fn percent_scales_both_sides_and_never_reaches_zero() {
        assert_eq!(
            dimensions_of(4000, 3000, ResizeOption::Percent { percent: 25 }),
            (1000, 750)
        );
        assert_eq!(
            dimensions_of(3, 3, ResizeOption::Percent { percent: 1 }),
            (1, 1)
        );
    }

    #[test]
    fn transparent_pixels_flatten_to_white() {
        assert_eq!(blend_channel_onto_white(0, 0), 255);
        assert_eq!(blend_channel_onto_white(10, 255), 10);
    }
}
