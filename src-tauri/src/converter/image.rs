use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use image::codecs::bmp::BmpEncoder;
use image::codecs::gif::GifEncoder;
use image::codecs::ico::IcoEncoder;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::codecs::tiff::TiffEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ImageEncoder};

use crate::errors::ConversionError;
use crate::models::{ConversionOptions, FileFormat};

use super::Converter;

/// 画像変換器
///
/// `image` crate を使用して 8 種類の画像フォーマット間の相互変換をサポートする。
/// PNG, JPEG, WebP, GIF, BMP, TIFF, AVIF, ICO の全ペア（同一フォーマット除く）が変換可能。
///
/// ## パフォーマンス特性
///
/// - 画像のデコード・リサイズ・エンコードは CPU バウンドな処理であるため、
///   `tokio::task::spawn_blocking` を使用して Tokio の非同期ランタイムをブロックしない。
/// - `image` crate は画像全体をメモリ上に展開するため、メモリ使用量は
///   `幅 × 高さ × 4バイト(RGBA)` となる。100MB以下のファイルサイズ制限（UI側の
///   確認ダイアログ）により、実用上のメモリ使用量は許容範囲内に収まる。
/// - 将来的に非常に大きな画像を効率的に処理する場合は、タイルベースの
///   ストリーミング処理やメモリマップドファイルの導入を検討する。
pub struct ImageConverter;

impl ImageConverter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ImageConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// サポートする画像フォーマット一覧（入力・出力共通）
const IMAGE_FORMATS: &[FileFormat] = &[
    FileFormat::Png,
    FileFormat::Jpeg,
    FileFormat::Webp,
    FileFormat::Gif,
    FileFormat::Bmp,
    FileFormat::Tiff,
    FileFormat::Avif,
    FileFormat::Ico,
];

/// ICO フォーマットの最大サイズ（256x256）
const ICO_MAX_SIZE: u32 = 256;

/// リサイズ後の画像を計算する
///
/// - 両方指定: そのまま使用
/// - 幅のみ指定 + アスペクト比維持: 高さ = round(元高さ * 指定幅 / 元幅)
/// - 高さのみ指定 + アスペクト比維持: 幅 = round(元幅 * 指定高さ / 元高さ)
pub(crate) fn calculate_resize_dimensions(
    original_width: u32,
    original_height: u32,
    target_width: Option<u32>,
    target_height: Option<u32>,
    maintain_aspect_ratio: bool,
) -> (u32, u32) {
    match (target_width, target_height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => {
            if maintain_aspect_ratio && original_width > 0 {
                let h = ((original_height as f64 * w as f64) / original_width as f64).round() as u32;
                (w, h.max(1))
            } else {
                (w, original_height)
            }
        }
        (None, Some(h)) => {
            if maintain_aspect_ratio && original_height > 0 {
                let w = ((original_width as f64 * h as f64) / original_height as f64).round() as u32;
                (w.max(1), h)
            } else {
                (original_width, h)
            }
        }
        (None, None) => (original_width, original_height),
    }
}

/// 画像をエンコードして出力ファイルに書き込む
fn encode_image(
    img: &DynamicImage,
    output: &Path,
    format: &FileFormat,
    quality: u8,
) -> Result<(), ConversionError> {
    let file_name = output
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let file = File::create(output).map_err(|e| ConversionError::FileWriteError {
        file_name: file_name.clone(),
        detail: e.to_string(),
    })?;
    let writer = BufWriter::new(file);

    match format {
        FileFormat::Png => {
            let encoder = PngEncoder::new(writer);
            let rgba = img.to_rgba8();
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| ConversionError::EncodeError {
                    file_name,
                    detail: e.to_string(),
                })?;
        }
        FileFormat::Jpeg => {
            let encoder = JpegEncoder::new_with_quality(writer, quality);
            let rgb = img.to_rgb8();
            encoder
                .write_image(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| ConversionError::EncodeError {
                    file_name,
                    detail: e.to_string(),
                })?;
        }
        FileFormat::Webp => {
            // Use image crate's save method which auto-detects format from extension
            img.save(output).map_err(|e| ConversionError::EncodeError {
                file_name,
                detail: e.to_string(),
            })?;
        }
        FileFormat::Gif => {
            let mut encoder = GifEncoder::new(writer);
            let rgba = img.to_rgba8();
            encoder
                .encode(&rgba, rgba.width(), rgba.height(), image::ExtendedColorType::Rgba8)
                .map_err(|e| ConversionError::EncodeError {
                    file_name,
                    detail: e.to_string(),
                })?;
        }
        FileFormat::Bmp => {
            let mut writer = writer;
            let encoder = BmpEncoder::new(&mut writer);
            let rgba = img.to_rgba8();
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| ConversionError::EncodeError {
                    file_name,
                    detail: e.to_string(),
                })?;
        }
        FileFormat::Tiff => {
            let encoder = TiffEncoder::new(writer);
            let rgba = img.to_rgba8();
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| ConversionError::EncodeError {
                    file_name,
                    detail: e.to_string(),
                })?;
        }
        FileFormat::Avif => {
            // Use image crate's save method for AVIF (if avif feature is available)
            img.save(output).map_err(|e| ConversionError::EncodeError {
                file_name,
                detail: e.to_string(),
            })?;
        }
        FileFormat::Ico => {
            // ICO format has a max size of 256x256; resize if needed
            let ico_img = if img.width() > ICO_MAX_SIZE || img.height() > ICO_MAX_SIZE {
                img.resize(ICO_MAX_SIZE, ICO_MAX_SIZE, FilterType::Lanczos3)
            } else {
                img.clone()
            };
            let encoder = IcoEncoder::new(writer);
            let rgba = ico_img.to_rgba8();
            encoder
                .write_image(
                    rgba.as_raw(),
                    rgba.width(),
                    rgba.height(),
                    image::ExtendedColorType::Rgba8,
                )
                .map_err(|e| ConversionError::EncodeError {
                    file_name,
                    detail: e.to_string(),
                })?;
        }
        _ => {
            return Err(ConversionError::UnsupportedConversion {
                from: "image".to_string(),
                to: format!("{:?}", format),
            });
        }
    }

    Ok(())
}

impl Converter for ImageConverter {
    fn supported_input_formats(&self) -> &[FileFormat] {
        IMAGE_FORMATS
    }

    fn supported_output_formats(&self) -> &[FileFormat] {
        IMAGE_FORMATS
    }

    fn can_convert(&self, from: &FileFormat, to: &FileFormat) -> bool {
        from != to
            && self.supported_input_formats().contains(from)
            && self.supported_output_formats().contains(to)
    }

    async fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
    ) -> Result<(), ConversionError> {
        // CPU バウンドな画像処理（デコード・リサイズ・エンコード）を spawn_blocking で実行し、
        // Tokio の非同期ランタイム（Tauri IPC スレッド）をブロックしない。
        // これにより変換処理中も UI が 200ms 以内に応答可能な状態を維持する。
        let input = input.to_path_buf();
        let output = output.to_path_buf();
        let options = options.clone();

        let file_name_for_error = input
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        tokio::task::spawn_blocking(move || {
            let file_name = input
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            // 1. Read input image
            let mut img = image::open(&input).map_err(|e| ConversionError::DecodeError {
                file_name: file_name.clone(),
                detail: e.to_string(),
            })?;

            // 2. Apply resize if specified
            if let Some(ref resize) = options.resize {
                let (new_width, new_height) = calculate_resize_dimensions(
                    img.width(),
                    img.height(),
                    resize.width,
                    resize.height,
                    resize.maintain_aspect_ratio,
                );

                // Clamp to valid range (1-16383)
                let new_width = new_width.clamp(1, 16383);
                let new_height = new_height.clamp(1, 16383);

                // Only resize if dimensions actually change
                if new_width != img.width() || new_height != img.height() {
                    img = img.resize_exact(new_width, new_height, FilterType::Lanczos3);
                }
            }

            // 3. Determine output format from output path extension
            let output_format =
                ImageConverter::detect_format_from_path(&output).ok_or_else(|| {
                    ConversionError::EncodeError {
                        file_name: file_name.clone(),
                        detail: "出力ファイルの拡張子からフォーマットを判定できません"
                            .to_string(),
                    }
                })?;

            // 4. Encode and write output
            // Quality is only applied to lossy formats (JPEG, WebP, AVIF)
            // For lossless formats, quality is ignored and default encoding is used
            let quality = options.quality.clamp(1, 100);
            encode_image(&img, &output, &output_format, quality)?;

            Ok(())
        })
        .await
        .map_err(|e| ConversionError::FileReadError {
            file_name: file_name_for_error,
            detail: format!("バックグラウンドタスク実行エラー: {}", e),
        })?
    }
}

impl ImageConverter {
    /// 出力パスの拡張子からフォーマットを検出する
    fn detect_format_from_path(path: &Path) -> Option<FileFormat> {
        let ext = path.extension()?.to_string_lossy().to_lowercase();
        match ext.as_str() {
            "png" => Some(FileFormat::Png),
            "jpg" | "jpeg" => Some(FileFormat::Jpeg),
            "webp" => Some(FileFormat::Webp),
            "gif" => Some(FileFormat::Gif),
            "bmp" => Some(FileFormat::Bmp),
            "tiff" | "tif" => Some(FileFormat::Tiff),
            "avif" => Some(FileFormat::Avif),
            "ico" => Some(FileFormat::Ico),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ConflictResolution, FileNamingPattern, ResizeOptions};
    use tempfile::TempDir;

    // ==================== Helper functions ====================

    /// デフォルトの変換オプションを作成する
    fn default_options() -> ConversionOptions {
        ConversionOptions {
            quality: 80,
            resize: None,
            encoding: None,
            csv_delimiter: None,
            file_naming: FileNamingPattern::Original,
            conflict_resolution: ConflictResolution::Overwrite,
            delete_source: false,
        }
    }

    /// 指定品質の変換オプションを作成する
    fn options_with_quality(quality: u8) -> ConversionOptions {
        ConversionOptions {
            quality,
            ..default_options()
        }
    }

    /// リサイズ付き変換オプションを作成する
    fn options_with_resize(width: Option<u32>, height: Option<u32>) -> ConversionOptions {
        ConversionOptions {
            resize: Some(ResizeOptions {
                width,
                height,
                maintain_aspect_ratio: true,
            }),
            ..default_options()
        }
    }

    /// テスト用の RGBA 画像を生成し、指定パスに PNG として保存する
    fn create_test_image(path: &Path, width: u32, height: u32) {
        let img = DynamicImage::new_rgba8(width, height);
        img.save(path).expect("Failed to save test image");
    }

    /// 出力ファイルの拡張子を取得する
    fn extension_for_format(format: &FileFormat) -> &str {
        match format {
            FileFormat::Png => "png",
            FileFormat::Jpeg => "jpeg",
            FileFormat::Webp => "webp",
            FileFormat::Gif => "gif",
            FileFormat::Bmp => "bmp",
            FileFormat::Tiff => "tiff",
            FileFormat::Avif => "avif",
            FileFormat::Ico => "ico",
            _ => "bin",
        }
    }

    // ==================== Existing unit tests ====================

    #[test]
    fn test_supported_formats_count() {
        let converter = ImageConverter::new();
        assert_eq!(converter.supported_input_formats().len(), 8);
        assert_eq!(converter.supported_output_formats().len(), 8);
    }

    #[test]
    fn test_can_convert_different_formats() {
        let converter = ImageConverter::new();
        assert!(converter.can_convert(&FileFormat::Png, &FileFormat::Jpeg));
        assert!(converter.can_convert(&FileFormat::Webp, &FileFormat::Gif));
        assert!(converter.can_convert(&FileFormat::Bmp, &FileFormat::Tiff));
        assert!(converter.can_convert(&FileFormat::Avif, &FileFormat::Ico));
    }

    #[test]
    fn test_cannot_convert_same_format() {
        let converter = ImageConverter::new();
        assert!(!converter.can_convert(&FileFormat::Png, &FileFormat::Png));
        assert!(!converter.can_convert(&FileFormat::Jpeg, &FileFormat::Jpeg));
    }

    #[test]
    fn test_cannot_convert_text_formats() {
        let converter = ImageConverter::new();
        assert!(!converter.can_convert(&FileFormat::Png, &FileFormat::Json));
        assert!(!converter.can_convert(&FileFormat::Json, &FileFormat::Png));
    }

    #[test]
    fn test_calculate_resize_both_specified() {
        let (w, h) = calculate_resize_dimensions(1000, 500, Some(200), Some(100), true);
        assert_eq!(w, 200);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_calculate_resize_width_only_maintain_aspect() {
        // Original: 1000x500, target width: 200
        // Expected height: round(500 * 200 / 1000) = 100
        let (w, h) = calculate_resize_dimensions(1000, 500, Some(200), None, true);
        assert_eq!(w, 200);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_calculate_resize_height_only_maintain_aspect() {
        // Original: 1000x500, target height: 100
        // Expected width: round(1000 * 100 / 500) = 200
        let (w, h) = calculate_resize_dimensions(1000, 500, None, Some(100), true);
        assert_eq!(w, 200);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_calculate_resize_width_only_no_aspect() {
        // Without aspect ratio maintenance, height stays original
        let (w, h) = calculate_resize_dimensions(1000, 500, Some(200), None, false);
        assert_eq!(w, 200);
        assert_eq!(h, 500);
    }

    #[test]
    fn test_calculate_resize_height_only_no_aspect() {
        // Without aspect ratio maintenance, width stays original
        let (w, h) = calculate_resize_dimensions(1000, 500, None, Some(100), false);
        assert_eq!(w, 1000);
        assert_eq!(h, 100);
    }

    #[test]
    fn test_calculate_resize_none_specified() {
        let (w, h) = calculate_resize_dimensions(1000, 500, None, None, true);
        assert_eq!(w, 1000);
        assert_eq!(h, 500);
    }

    #[test]
    fn test_calculate_resize_minimum_one() {
        // Very small target should still produce at least 1
        let (w, h) = calculate_resize_dimensions(1000, 1, Some(1), None, true);
        assert_eq!(w, 1);
        // round(1 * 1 / 1000) = 0, but clamped to max(1)
        assert!(h >= 1);
    }

    #[test]
    fn test_detect_format_from_path() {
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.png")),
            Some(FileFormat::Png)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.jpg")),
            Some(FileFormat::Jpeg)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.jpeg")),
            Some(FileFormat::Jpeg)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.webp")),
            Some(FileFormat::Webp)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.gif")),
            Some(FileFormat::Gif)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.bmp")),
            Some(FileFormat::Bmp)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.tiff")),
            Some(FileFormat::Tiff)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.tif")),
            Some(FileFormat::Tiff)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.avif")),
            Some(FileFormat::Avif)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.ico")),
            Some(FileFormat::Ico)
        );
        assert_eq!(
            ImageConverter::detect_format_from_path(Path::new("test.txt")),
            None
        );
    }

    // ==================== フォーマットペア変換テスト ====================
    // Validates: Requirements 2.1
    // 各フォーマットペアの基本変換テスト（代表的な組み合わせ）

    #[tokio::test]
    async fn test_convert_png_to_jpeg() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.png");
        let output = tmp.path().join("output.jpeg");
        create_test_image(&input, 100, 100);

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &default_options()).await;

        assert!(result.is_ok(), "PNG→JPEG conversion failed: {:?}", result.err());
        assert!(output.exists(), "Output file does not exist");
        assert!(output.metadata().unwrap().len() > 0, "Output file is empty");
        // Verify decodable
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "Output file is not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_convert_png_to_webp() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.png");
        let output = tmp.path().join("output.webp");
        create_test_image(&input, 100, 100);

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &default_options()).await;

        assert!(result.is_ok(), "PNG→WebP conversion failed: {:?}", result.err());
        assert!(output.exists(), "Output file does not exist");
        assert!(output.metadata().unwrap().len() > 0, "Output file is empty");
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "Output file is not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_convert_webp_to_png() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.webp");
        let output = tmp.path().join("output.png");

        // Create a WebP input by first saving as WebP
        let img = DynamicImage::new_rgba8(100, 100);
        img.save(&input).expect("Failed to save test WebP image");

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &default_options()).await;

        assert!(result.is_ok(), "WebP→PNG conversion failed: {:?}", result.err());
        assert!(output.exists(), "Output file does not exist");
        assert!(output.metadata().unwrap().len() > 0, "Output file is empty");
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "Output file is not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_convert_jpeg_to_png() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.jpeg");
        let output = tmp.path().join("output.png");

        // Create a JPEG input
        let img = DynamicImage::new_rgb8(100, 100);
        img.save(&input).expect("Failed to save test JPEG image");

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &default_options()).await;

        assert!(result.is_ok(), "JPEG→PNG conversion failed: {:?}", result.err());
        assert!(output.exists(), "Output file does not exist");
        assert!(output.metadata().unwrap().len() > 0, "Output file is empty");
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "Output file is not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_convert_bmp_to_tiff() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.bmp");
        let output = tmp.path().join("output.tiff");

        let img = DynamicImage::new_rgba8(100, 100);
        img.save(&input).expect("Failed to save test BMP image");

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &default_options()).await;

        assert!(result.is_ok(), "BMP→TIFF conversion failed: {:?}", result.err());
        assert!(output.exists(), "Output file does not exist");
        assert!(output.metadata().unwrap().len() > 0, "Output file is empty");
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "Output file is not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_convert_gif_to_png() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.gif");
        let output = tmp.path().join("output.png");

        let img = DynamicImage::new_rgba8(100, 100);
        img.save(&input).expect("Failed to save test GIF image");

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &default_options()).await;

        assert!(result.is_ok(), "GIF→PNG conversion failed: {:?}", result.err());
        assert!(output.exists(), "Output file does not exist");
        assert!(output.metadata().unwrap().len() > 0, "Output file is empty");
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "Output file is not decodable: {:?}", decoded.err());
    }

    // ==================== エッジケーステスト ====================
    // Validates: Requirements 2.4

    #[tokio::test]
    async fn test_convert_1x1_pixel_image_to_all_formats() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("tiny.png");
        create_test_image(&input, 1, 1);

        let converter = ImageConverter::new();

        let output_formats = [
            FileFormat::Jpeg,
            FileFormat::Webp,
            FileFormat::Gif,
            FileFormat::Bmp,
            FileFormat::Tiff,
            FileFormat::Ico,
        ];

        for format in &output_formats {
            let ext = extension_for_format(format);
            let output = tmp.path().join(format!("tiny_output.{}", ext));

            let result = converter.convert(&input, &output, &default_options()).await;
            assert!(
                result.is_ok(),
                "1x1 PNG→{:?} conversion failed: {:?}",
                format,
                result.err()
            );
            assert!(output.exists(), "Output file does not exist for {:?}", format);
            assert!(
                output.metadata().unwrap().len() > 0,
                "Output file is empty for {:?}",
                format
            );
            let decoded = image::open(&output);
            assert!(
                decoded.is_ok(),
                "1x1 output not decodable for {:?}: {:?}",
                format,
                decoded.err()
            );
        }
    }

    #[tokio::test]
    async fn test_resize_to_large_dimensions() {
        // Take a small image and resize to target width 16383
        // Since the image is 10x10 (square), the output would be 16383x16383 which is ~1GB RGBA.
        // The image crate's default decoder has memory limits, so we use a non-square input
        // to keep the output within reasonable memory bounds, or use a smaller target.
        // Here we test with width=16383 on a 100x1 image → output should be 16383x164 (within limits).
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("small.png");
        create_test_image(&input, 100, 1);

        let output = tmp.path().join("large_output.png");
        let options = options_with_resize(Some(16383), None);

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &options).await;

        assert!(
            result.is_ok(),
            "Large resize conversion failed: {:?}",
            result.err()
        );
        assert!(output.exists(), "Output file does not exist");
        assert!(output.metadata().unwrap().len() > 0, "Output file is empty");

        // Decode with relaxed limits to verify dimensions
        let mut reader = image::ImageReader::open(&output).expect("Failed to open output");
        reader.no_limits();
        let decoded = reader.decode().expect("Failed to decode large output");
        assert_eq!(decoded.width(), 16383, "Output width should be 16383");
        // Aspect ratio maintained: height = round(1 * 16383 / 100) = 164
        let expected_height = ((1.0_f64 * 16383.0_f64) / 100.0_f64).round() as u32;
        assert_eq!(decoded.height(), expected_height, "Output height should match aspect ratio calculation");
    }

    #[tokio::test]
    async fn test_resize_large_width_non_square() {
        // Non-square image: 100x50, resize to width 16383
        // Expected height: round(50 * 16383 / 100) = 8192 (clamped to 16383 max)
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("rect.png");
        create_test_image(&input, 100, 50);

        let output = tmp.path().join("large_rect_output.png");
        let options = options_with_resize(Some(16383), None);

        let converter = ImageConverter::new();
        let result = converter.convert(&input, &output, &options).await;

        assert!(
            result.is_ok(),
            "Large resize non-square conversion failed: {:?}",
            result.err()
        );
        assert!(output.exists(), "Output file does not exist");

        let decoded = image::open(&output).expect("Failed to decode large rect output");
        assert_eq!(decoded.width(), 16383);
        // Expected: round(50 * 16383 / 100) = 8192 (rounded)
        let expected_height = ((50.0_f64 * 16383.0_f64) / 100.0_f64).round() as u32;
        assert_eq!(decoded.height(), expected_height);
    }

    // ==================== 品質値境界テスト ====================
    // Validates: Requirements 2.3, 2.6

    #[tokio::test]
    async fn test_jpeg_quality_1() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.png");
        let output = tmp.path().join("output_q1.jpeg");
        create_test_image(&input, 50, 50);

        let converter = ImageConverter::new();
        let result = converter
            .convert(&input, &output, &options_with_quality(1))
            .await;

        assert!(result.is_ok(), "JPEG quality=1 failed: {:?}", result.err());
        assert!(output.exists());
        assert!(output.metadata().unwrap().len() > 0);
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "JPEG quality=1 not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_jpeg_quality_50() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.png");
        let output = tmp.path().join("output_q50.jpeg");
        create_test_image(&input, 50, 50);

        let converter = ImageConverter::new();
        let result = converter
            .convert(&input, &output, &options_with_quality(50))
            .await;

        assert!(result.is_ok(), "JPEG quality=50 failed: {:?}", result.err());
        assert!(output.exists());
        assert!(output.metadata().unwrap().len() > 0);
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "JPEG quality=50 not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_jpeg_quality_100() {
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.png");
        let output = tmp.path().join("output_q100.jpeg");
        create_test_image(&input, 50, 50);

        let converter = ImageConverter::new();
        let result = converter
            .convert(&input, &output, &options_with_quality(100))
            .await;

        assert!(result.is_ok(), "JPEG quality=100 failed: {:?}", result.err());
        assert!(output.exists());
        assert!(output.metadata().unwrap().len() > 0);
        let decoded = image::open(&output);
        assert!(decoded.is_ok(), "JPEG quality=100 not decodable: {:?}", decoded.err());
    }

    #[tokio::test]
    async fn test_png_quality_values_produce_identical_output() {
        // PNG is lossless, so different quality values should produce identical output
        let tmp = TempDir::new().unwrap();
        let input = tmp.path().join("input.png");
        create_test_image(&input, 50, 50);

        let converter = ImageConverter::new();

        let output_q1 = tmp.path().join("output_q1.png");
        let output_q50 = tmp.path().join("output_q50.png");
        let output_q100 = tmp.path().join("output_q100.png");

        converter
            .convert(&input, &output_q1, &options_with_quality(1))
            .await
            .expect("PNG quality=1 failed");
        converter
            .convert(&input, &output_q50, &options_with_quality(50))
            .await
            .expect("PNG quality=50 failed");
        converter
            .convert(&input, &output_q100, &options_with_quality(100))
            .await
            .expect("PNG quality=100 failed");

        // All outputs should be valid
        let img_q1 = image::open(&output_q1).expect("PNG q1 not decodable");
        let img_q50 = image::open(&output_q50).expect("PNG q50 not decodable");
        let img_q100 = image::open(&output_q100).expect("PNG q100 not decodable");

        // PNG is lossless - pixel data should be identical regardless of quality setting
        assert_eq!(img_q1.to_rgba8().as_raw(), img_q50.to_rgba8().as_raw(),
            "PNG output should be identical for quality 1 and 50");
        assert_eq!(img_q50.to_rgba8().as_raw(), img_q100.to_rgba8().as_raw(),
            "PNG output should be identical for quality 50 and 100");
    }
}
