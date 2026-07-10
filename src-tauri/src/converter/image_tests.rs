//! Property-based tests for image conversion
//!
//! Feature: universal-file-converter

use proptest::prelude::*;
use std::path::Path;
use tempfile::TempDir;

use image::{DynamicImage, ImageBuffer, Rgba};

use crate::converter::image::{calculate_resize_dimensions, ImageConverter};
use crate::converter::Converter;
use crate::models::{
    ConversionOptions, ConflictResolution, FileFormat, FileNamingPattern,
};

/// Helper: Create default conversion options with specified quality
fn make_options(quality: u8) -> ConversionOptions {
    ConversionOptions {
        quality,
        resize: None,
        encoding: None,
        csv_delimiter: None,
        file_naming: FileNamingPattern::Original,
        conflict_resolution: ConflictResolution::Overwrite,
        delete_source: false,
    }
}

/// Helper: Generate a random RGBA image and save it as PNG to the given path
fn create_test_image(path: &Path, width: u32, height: u32, pixels: &[u8]) {
    let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
    for (i, pixel) in img.pixels_mut().enumerate() {
        let base = (i * 4) % pixels.len().max(4);
        let r = pixels.get(base).copied().unwrap_or(128);
        let g = pixels.get(base + 1).copied().unwrap_or(64);
        let b = pixels.get(base + 2).copied().unwrap_or(32);
        let a = pixels.get(base + 3).copied().unwrap_or(255);
        *pixel = Rgba([r, g, b, a]);
    }
    let dynamic = DynamicImage::ImageRgba8(img);
    dynamic.save(path).expect("Failed to save test image");
}

/// Strategy for generating small image dimensions (8-64 pixels)
fn image_dimension_strategy() -> impl Strategy<Value = u32> {
    8u32..=64u32
}

/// Strategy for generating output formats (excluding AVIF due to encoding complexity in tests)
fn output_format_strategy() -> impl Strategy<Value = FileFormat> {
    prop_oneof![
        Just(FileFormat::Png),
        Just(FileFormat::Jpeg),
        Just(FileFormat::Webp),
        Just(FileFormat::Gif),
        Just(FileFormat::Bmp),
        Just(FileFormat::Tiff),
        Just(FileFormat::Ico),
    ]
}

/// Strategy for generating lossless output formats
fn lossless_format_strategy() -> impl Strategy<Value = FileFormat> {
    prop_oneof![
        Just(FileFormat::Png),
        Just(FileFormat::Gif),
        Just(FileFormat::Bmp),
        Just(FileFormat::Tiff),
        Just(FileFormat::Ico),
    ]
}

/// Get file extension for a format
fn format_extension(format: &FileFormat) -> &'static str {
    match format {
        FileFormat::Png => "png",
        FileFormat::Jpeg => "jpg",
        FileFormat::Webp => "webp",
        FileFormat::Gif => "gif",
        FileFormat::Bmp => "bmp",
        FileFormat::Tiff => "tiff",
        FileFormat::Avif => "avif",
        FileFormat::Ico => "ico",
        _ => "bin",
    }
}

// ============================================================================
// Property 1: 画像フォーマット変換の整合性
// Feature: universal-file-converter, Property 1: 画像フォーマット変換の整合性
// **Validates: Requirements 2.1**
//
// For any valid image file and any supported output format combination,
// the conversion engine produces a file that is valid for the output format
// and decodable.
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_image_format_conversion_produces_decodable_output(
        width in image_dimension_strategy(),
        height in image_dimension_strategy(),
        output_format in output_format_strategy(),
        pixel_seed in proptest::collection::vec(any::<u8>(), 256..=1024),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let tmp_dir = TempDir::new().unwrap();
            let input_path = tmp_dir.path().join("input.png");
            let ext = format_extension(&output_format);
            let output_path = tmp_dir.path().join(format!("output.{}", ext));

            // Create a test image as PNG input
            create_test_image(&input_path, width, height, &pixel_seed);

            // Convert to target format
            let converter = ImageConverter::new();
            let options = make_options(80);

            let conv_result = converter.convert(&input_path, &output_path, &options).await;
            if let Err(e) = conv_result {
                return Err(format!("Conversion failed: {:?}", e));
            }

            // Verify output file exists and is decodable
            if !output_path.exists() {
                return Err("Output file does not exist".to_string());
            }

            match image::open(&output_path) {
                Ok(img) => {
                    if img.width() == 0 || img.height() == 0 {
                        return Err("Decoded image has zero dimensions".to_string());
                    }
                    Ok(())
                }
                Err(e) => Err(format!("Output file is not decodable: {:?}", e)),
            }
        });
        prop_assert!(result.is_ok(), "{}", result.unwrap_err());
    }
}

// ============================================================================
// Property 2: リサイズ処理の寸法正確性
// Feature: universal-file-converter, Property 2: リサイズ処理の寸法正確性
// **Validates: Requirements 2.4**
//
// For any image (width W, height H) and valid resize parameters,
// when maintain_aspect_ratio is enabled, output dimensions match the expected
// calculation formula.
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_resize_width_only_with_aspect_ratio(
        original_width in 1u32..=200,
        original_height in 1u32..=200,
        target_width in 1u32..=200,
    ) {
        let (out_w, out_h) = calculate_resize_dimensions(
            original_width,
            original_height,
            Some(target_width),
            None,
            true,
        );

        // Width should match target
        prop_assert_eq!(out_w, target_width);

        // Height should be round(H * TW / W), minimum 1
        let expected_h = ((original_height as f64 * target_width as f64) / original_width as f64)
            .round() as u32;
        let expected_h = expected_h.max(1);
        prop_assert_eq!(out_h, expected_h,
            "Expected height {} for original {}x{} with target_width {}",
            expected_h, original_width, original_height, target_width);
    }

    #[test]
    fn prop_resize_height_only_with_aspect_ratio(
        original_width in 1u32..=200,
        original_height in 1u32..=200,
        target_height in 1u32..=200,
    ) {
        let (out_w, out_h) = calculate_resize_dimensions(
            original_width,
            original_height,
            None,
            Some(target_height),
            true,
        );

        // Height should match target
        prop_assert_eq!(out_h, target_height);

        // Width should be round(W * TH / H), minimum 1
        let expected_w = ((original_width as f64 * target_height as f64) / original_height as f64)
            .round() as u32;
        let expected_w = expected_w.max(1);
        prop_assert_eq!(out_w, expected_w,
            "Expected width {} for original {}x{} with target_height {}",
            expected_w, original_width, original_height, target_height);
    }

    #[test]
    fn prop_resize_both_specified(
        original_width in 1u32..=200,
        original_height in 1u32..=200,
        target_width in 1u32..=200,
        target_height in 1u32..=200,
    ) {
        let (out_w, out_h) = calculate_resize_dimensions(
            original_width,
            original_height,
            Some(target_width),
            Some(target_height),
            true, // aspect ratio flag is ignored when both are specified
        );

        // Both should match specified values exactly
        prop_assert_eq!(out_w, target_width);
        prop_assert_eq!(out_h, target_height);
    }
}

// ============================================================================
// Property 3: 可逆フォーマットにおける品質設定の無視
// Feature: universal-file-converter, Property 3: 可逆フォーマットにおける品質設定の無視
// **Validates: Requirements 2.6**
//
// For any image and lossless output format (PNG, GIF, BMP, TIFF, ICO) and
// any quality value (1-100), the output image pixel data is identical
// regardless of quality value.
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn prop_lossless_format_ignores_quality(
        width in 8u32..=32,
        height in 8u32..=32,
        lossless_format in lossless_format_strategy(),
        quality_a in 1u8..=100,
        quality_b in 1u8..=100,
        pixel_seed in proptest::collection::vec(any::<u8>(), 256..=512),
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let tmp_dir = TempDir::new().unwrap();
            let input_path = tmp_dir.path().join("input.png");
            let ext = format_extension(&lossless_format);
            let output_a_path = tmp_dir.path().join(format!("output_a.{}", ext));
            let output_b_path = tmp_dir.path().join(format!("output_b.{}", ext));

            // Create a test image
            create_test_image(&input_path, width, height, &pixel_seed);

            let converter = ImageConverter::new();

            // Convert with quality_a
            let options_a = make_options(quality_a);
            if let Err(e) = converter.convert(&input_path, &output_a_path, &options_a).await {
                return Err(format!("Conversion with quality_a={} failed: {:?}", quality_a, e));
            }

            // Convert with quality_b
            let options_b = make_options(quality_b);
            if let Err(e) = converter.convert(&input_path, &output_b_path, &options_b).await {
                return Err(format!("Conversion with quality_b={} failed: {:?}", quality_b, e));
            }

            // Decode both outputs and compare pixel data
            let img_a = image::open(&output_a_path)
                .map_err(|e| format!("Failed to open output_a: {:?}", e))?;
            let img_b = image::open(&output_b_path)
                .map_err(|e| format!("Failed to open output_b: {:?}", e))?;

            let pixels_a = img_a.to_rgba8();
            let pixels_b = img_b.to_rgba8();

            // Dimensions should be the same
            if pixels_a.width() != pixels_b.width() {
                return Err(format!(
                    "Width mismatch for {:?} format: {} vs {}",
                    lossless_format, pixels_a.width(), pixels_b.width()
                ));
            }
            if pixels_a.height() != pixels_b.height() {
                return Err(format!(
                    "Height mismatch for {:?} format: {} vs {}",
                    lossless_format, pixels_a.height(), pixels_b.height()
                ));
            }

            // Pixel data should be identical
            if pixels_a.as_raw() != pixels_b.as_raw() {
                return Err(format!(
                    "Pixel data differs for {:?} format with quality {} vs {}",
                    lossless_format, quality_a, quality_b
                ));
            }

            Ok(())
        });
        prop_assert!(result.is_ok(), "{}", result.unwrap_err());
    }
}
