use std::path::Path;

use anyhow::{Context, Result};
use image::imageops::FilterType;
use palette::{IntoColor, Lab, Srgb};

use crate::color::Color;

/// A color extracted from the image with its cluster weight.
#[derive(Debug, Clone)]
pub struct ExtractedColor {
    pub color: Color,
    pub weight: f32,
}

const MAX_DIM: u32 = 256;

/// Load an image, resize to fit within 256x256 (preserving aspect ratio),
/// and convert all pixels to CIELAB space.
pub fn load_and_prepare(path: &Path) -> Result<Vec<Lab>> {
    let img = image::open(path).with_context(|| {
        if !path.exists() {
            format!("file not found: {}", path.display())
        } else {
            format!(
                "unsupported or corrupt image: {}. Supported formats: PNG, JPEG, WebP, BMP, TIFF, GIF",
                path.display()
            )
        }
    })?;

    let img = if img.width() > MAX_DIM || img.height() > MAX_DIM {
        img.resize(MAX_DIM, MAX_DIM, FilterType::Lanczos3)
    } else {
        img
    };
    let rgb_img = img.to_rgb8();

    let pixels: Vec<Lab> = rgb_img
        .pixels()
        .map(|p| {
            let srgb: Srgb<f32> = Srgb::new(p[0], p[1], p[2]).into_format();
            srgb.into_color()
        })
        .collect();

    Ok(pixels)
}

/// Run K-means on LAB pixels to extract dominant colors.
pub fn extract_colors(_pixels: &[Lab], _k: usize) -> Vec<ExtractedColor> {
    todo!("Ticket 4: K-means color extraction")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join(name)
    }

    #[test]
    fn load_4x4_png() {
        // Create a 4x4 test PNG with known colors
        let path = fixture_path("4x4_test.png");
        create_test_image(&path, 4, 4);

        let pixels = load_and_prepare(&path).unwrap();
        // 4x4 image is below 256x256 so it stays the same size
        assert_eq!(pixels.len(), 16);
    }

    #[test]
    fn load_large_image_resizes() {
        // Create a 512x512 image that should be resized to 256x256
        let path = fixture_path("512x512_test.png");
        create_test_image(&path, 512, 512);

        let pixels = load_and_prepare(&path).unwrap();
        assert_eq!(pixels.len(), 256 * 256);
    }

    #[test]
    fn load_nonsquare_preserves_aspect_ratio() {
        // Create a 512x256 image — should resize to 256x128
        let path = fixture_path("512x256_test.png");
        create_test_image(&path, 512, 256);

        let pixels = load_and_prepare(&path).unwrap();
        assert_eq!(pixels.len(), 256 * 128);
    }

    #[test]
    fn load_file_not_found() {
        let result = load_and_prepare(Path::new("/nonexistent/image.png"));
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("file not found") || err.contains("No such file"),
            "expected file-not-found error, got: {err}"
        );
    }

    #[test]
    fn load_unsupported_format() {
        // Write a non-image file
        let path = fixture_path("not_an_image.txt");
        std::fs::write(&path, "this is not an image").unwrap();

        let result = load_and_prepare(&path);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("unsupported") || err.contains("Unsupported"),
            "expected unsupported format error, got: {err}"
        );
    }

    #[test]
    fn pixels_are_valid_lab() {
        let path = fixture_path("4x4_test.png");
        create_test_image(&path, 4, 4);

        let pixels = load_and_prepare(&path).unwrap();
        for lab in &pixels {
            // L should be in [0, 100] range for valid colors
            assert!(lab.l >= 0.0 && lab.l <= 100.0, "L out of range: {}", lab.l);
        }
    }

    fn create_test_image(path: &Path, width: u32, height: u32) {
        let mut img = image::RgbImage::new(width, height);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = ((x * 255) / width.max(1)) as u8;
            let g = ((y * 255) / height.max(1)) as u8;
            let b = 128u8;
            *pixel = image::Rgb([r, g, b]);
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        img.save(path).unwrap();
    }
}
