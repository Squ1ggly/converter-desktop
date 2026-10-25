use image::{DynamicImage, ImageBuffer, Pixel};
use moxcms::{ColorProfile, DataColorSpace, Layout, TransformExecutor, TransformOptions};

// Pixels are converted to sRGB as they are read, so every output looks right
// in any app, including formats and viewers that ignore colour profiles. A
// Display P3 photo would otherwise look washed out once its profile is gone.
// Greyscale and CMYK profiles are left alone.
pub fn convert_to_srgb(image: DynamicImage, icc_profile: Option<&[u8]>) -> DynamicImage {
    let Some(icc_profile) = icc_profile else {
        return image;
    };
    let source_profile = match ColorProfile::new_from_slice(icc_profile) {
        Ok(source_profile) => source_profile,
        Err(error) => {
            log::warn!("ignored an unreadable colour profile: {error}");
            return image;
        }
    };
    if source_profile.color_space != DataColorSpace::Rgb {
        return image;
    }

    let converted_image = match image {
        DynamicImage::ImageRgb8(pixels) => {
            convert_pixels(pixels, &source_profile, Layout::Rgb, transform_8bit).into()
        }
        DynamicImage::ImageRgba8(pixels) => {
            convert_pixels(pixels, &source_profile, Layout::Rgba, transform_8bit).into()
        }
        DynamicImage::ImageRgb16(pixels) => {
            convert_pixels(pixels, &source_profile, Layout::Rgb, transform_16bit).into()
        }
        DynamicImage::ImageRgba16(pixels) => {
            convert_pixels(pixels, &source_profile, Layout::Rgba, transform_16bit).into()
        }
        other_image => other_image,
    };
    converted_image
}

type Transform<Channel> = std::sync::Arc<dyn TransformExecutor<Channel> + Send + Sync>;

fn transform_8bit(source_profile: &ColorProfile, layout: Layout) -> Option<Transform<u8>> {
    let srgb_profile = ColorProfile::new_srgb();
    let transform = source_profile
        .create_transform_8bit(layout, &srgb_profile, layout, TransformOptions::default())
        .map_err(|error| log::warn!("couldn't convert a colour profile to sRGB: {error}"))
        .ok()?;
    Some(transform)
}

fn transform_16bit(source_profile: &ColorProfile, layout: Layout) -> Option<Transform<u16>> {
    let srgb_profile = ColorProfile::new_srgb();
    let transform = source_profile
        .create_transform_16bit(layout, &srgb_profile, layout, TransformOptions::default())
        .map_err(|error| log::warn!("couldn't convert a colour profile to sRGB: {error}"))
        .ok()?;
    Some(transform)
}

// Leaves the pixels as they are if the profile can't be converted.
fn convert_pixels<PixelType, Channel>(
    pixels: ImageBuffer<PixelType, Vec<Channel>>,
    source_profile: &ColorProfile,
    layout: Layout,
    create_transform: fn(&ColorProfile, Layout) -> Option<Transform<Channel>>,
) -> ImageBuffer<PixelType, Vec<Channel>>
where
    PixelType: Pixel<Subpixel = Channel>,
    Channel: Copy + Default,
{
    let Some(transform) = create_transform(source_profile, layout) else {
        return pixels;
    };

    let mut converted_pixels = ImageBuffer::new(pixels.width(), pixels.height());
    if let Err(error) = transform.transform(pixels.as_raw(), &mut converted_pixels) {
        log::warn!("couldn't convert pixels to sRGB: {error}");
        return pixels;
    }
    converted_pixels
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    fn display_p3_profile() -> Vec<u8> {
        ColorProfile::new_display_p3().encode().unwrap()
    }

    // Display P3 (204, 102, 51) is sRGB (219, 94, 31) by the standard
    // P3-D65 to sRGB matrix. Left unconverted it would stay (204, 102, 51).
    #[test]
    fn display_p3_colours_become_their_srgb_equivalents() {
        let p3_image = DynamicImage::ImageRgb8(RgbImage::from_pixel(2, 2, Rgb([204, 102, 51])));

        let converted_image = convert_to_srgb(p3_image, Some(&display_p3_profile()));

        let [red, green, blue] = converted_image.to_rgb8().get_pixel(0, 0).0;
        let expected = [219, 94, 31];
        for (channel, expected_channel) in [red, green, blue].into_iter().zip(expected) {
            assert!(
                channel.abs_diff(expected_channel) <= 2,
                "got {:?}, expected about {expected:?}",
                [red, green, blue]
            );
        }
    }

    #[test]
    fn images_without_a_profile_are_unchanged() {
        let image = DynamicImage::ImageRgb8(RgbImage::from_pixel(1, 1, Rgb([204, 102, 51])));

        let converted_image = convert_to_srgb(image, None);

        assert_eq!(converted_image.to_rgb8().get_pixel(0, 0).0, [204, 102, 51]);
    }

    #[test]
    fn an_unreadable_profile_is_ignored() {
        let image = DynamicImage::ImageRgb8(RgbImage::from_pixel(1, 1, Rgb([204, 102, 51])));

        let converted_image = convert_to_srgb(image, Some(b"not a profile"));

        assert_eq!(converted_image.to_rgb8().get_pixel(0, 0).0, [204, 102, 51]);
    }
}
