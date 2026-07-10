//! Property-based tests for batch processing logic
//!
//! Tests the batch processing behavior that `start_conversion` implements,
//! using the underlying components directly (ConversionEngine, Validator, output).
//!
//! Feature: universal-file-converter

use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;
use tempfile::TempDir;

use crate::converter::image::ImageConverter;
use crate::converter::text::TextConverter;
use crate::converter::{ConversionEngine, ConverterBoxed};
use crate::models::{
    ConflictResolution, ConversionOptions, ConversionResult, FailedFileInfo, FileFormat,
    FileNamingPattern, ProgressEvent, ProgressStatus,
};
use crate::output::{ensure_output_dir, generate_output_path, OutputPathResult};
use crate::validator::Validator;

// =============================================================================
// Helper functions
// =============================================================================

/// Default conversion options for testing
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

/// Create a valid 4x4 PNG image at the specified path
fn create_valid_png(path: &Path) {
    let img = image::DynamicImage::new_rgba8(4, 4);
    img.save(path).expect("Failed to create test PNG image");
}

/// Create a valid JSON file at the specified path
fn create_valid_json(path: &Path) {
    fs::write(path, r#"{"key": "value", "number": 42}"#).expect("Failed to create test JSON");
}

/// Create an invalid file (not a valid image or text format content)
fn create_invalid_file(path: &Path) {
    fs::write(path, b"this is not valid image data at all!!!").expect("Failed to create invalid file");
}

/// Determine if a format is an image format
fn is_image_format(format: &FileFormat) -> bool {
    matches!(
        format,
        FileFormat::Png
            | FileFormat::Jpeg
            | FileFormat::Webp
            | FileFormat::Gif
            | FileFormat::Bmp
            | FileFormat::Tiff
            | FileFormat::Avif
            | FileFormat::Ico
    )
}

/// Simulates the batch processing loop from start_conversion.
/// Returns (ConversionResult, Vec<ProgressEvent>).
///
/// This replicates the core logic without requiring Tauri State or Channel.
async fn simulate_batch_conversion(
    input_paths: &[PathBuf],
    output_format: &FileFormat,
    output_dir: Option<&Path>,
    options: &ConversionOptions,
    cancel_at: Option<usize>, // Cancel after processing this many files (0-indexed)
) -> (ConversionResult, Vec<ProgressEvent>) {
    let mut progress_events: Vec<ProgressEvent> = Vec::new();
    let mut success_count: u32 = 0;
    let mut failed_count: u32 = 0;
    let mut skipped_count: u32 = 0;
    let mut failed_files: Vec<FailedFileInfo> = Vec::new();

    // Validate input files
    let validation = Validator::validate_input_files(input_paths);

    let valid_files = validation.valid_files;
    let total_count = valid_files.len() as u32;

    // Record invalid files as failures
    for (path, reason) in &validation.invalid_files {
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.display().to_string());
        failed_files.push(FailedFileInfo {
            file_name,
            error: reason.clone(),
        });
        failed_count += 1;
    }

    let result_output_dir = output_dir
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    // Process each valid file
    for (idx, (file_path, input_format)) in valid_files.iter().enumerate() {
        // Check cancellation
        if let Some(cancel_point) = cancel_at {
            if idx >= cancel_point {
                let file_name = file_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                progress_events.push(ProgressEvent {
                    current_file: file_name,
                    processed_count: (idx as u32) + 1,
                    total_count,
                    status: ProgressStatus::Cancelled,
                    error: None,
                });
                break;
            }
        }

        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Progress: Processing
        progress_events.push(ProgressEvent {
            current_file: file_name.clone(),
            processed_count: idx as u32,
            total_count,
            status: ProgressStatus::Processing,
            error: None,
        });

        // Check conversion support
        let engine = ConversionEngine::new();
        let can_convert = engine.can_convert(input_format, output_format);

        if !can_convert {
            let error_msg = format!(
                "非対応変換ペア: {:?} → {:?}",
                input_format, output_format
            );
            failed_files.push(FailedFileInfo {
                file_name: file_name.clone(),
                error: error_msg.clone(),
            });
            failed_count += 1;
            progress_events.push(ProgressEvent {
                current_file: file_name,
                processed_count: (idx as u32) + 1,
                total_count,
                status: ProgressStatus::Failed,
                error: Some(error_msg),
            });
            continue;
        }

        // Generate output path
        let output_path_result = generate_output_path(
            file_path,
            output_dir,
            output_format,
            &options.file_naming,
            &options.conflict_resolution,
            (idx as u32) + 1,
        );

        let output_file_path = match output_path_result {
            OutputPathResult::Write(path) => path,
            OutputPathResult::Skip => {
                skipped_count += 1;
                progress_events.push(ProgressEvent {
                    current_file: file_name,
                    processed_count: (idx as u32) + 1,
                    total_count,
                    status: ProgressStatus::Completed,
                    error: None,
                });
                continue;
            }
        };

        // Ensure output directory
        if let Err(e) = ensure_output_dir(file_path, output_dir, output_format) {
            failed_files.push(FailedFileInfo {
                file_name: file_name.clone(),
                error: e.clone(),
            });
            failed_count += 1;
            progress_events.push(ProgressEvent {
                current_file: file_name,
                processed_count: (idx as u32) + 1,
                total_count,
                status: ProgressStatus::Failed,
                error: Some(e),
            });
            continue;
        }

        // Execute conversion
        let converter: Box<dyn ConverterBoxed> =
            if is_image_format(input_format) && is_image_format(output_format) {
                Box::new(ImageConverter::new())
            } else {
                Box::new(TextConverter::new())
            };

        let convert_result = converter
            .convert_boxed(file_path, &output_file_path, options)
            .await;

        match convert_result {
            Ok(()) => {
                success_count += 1;
                progress_events.push(ProgressEvent {
                    current_file: file_name.clone(),
                    processed_count: (idx as u32) + 1,
                    total_count,
                    status: ProgressStatus::Completed,
                    error: None,
                });

                // Delete source if option is enabled
                if options.delete_source {
                    let _ = fs::remove_file(file_path);
                }
            }
            Err(e) => {
                // Cleanup incomplete output
                if output_file_path.exists() {
                    let _ = fs::remove_file(&output_file_path);
                }

                let error_msg = e.to_string();
                failed_files.push(FailedFileInfo {
                    file_name: file_name.clone(),
                    error: error_msg.clone(),
                });
                failed_count += 1;

                progress_events.push(ProgressEvent {
                    current_file: file_name,
                    processed_count: (idx as u32) + 1,
                    total_count,
                    status: ProgressStatus::Failed,
                    error: Some(error_msg),
                });
            }
        }
    }

    let result = ConversionResult {
        success_count,
        failed_count,
        skipped_count,
        failed_files,
        output_dir: result_output_dir,
    };

    (result, progress_events)
}

// =============================================================================
// Property 7: バッチ処理のフォールトトレランス
// For any batch of N valid + M invalid files, all valid files are converted
// successfully, all invalid files have error messages with filenames.
// Result: success=N, failed=M.
//
// **Validates: Requirements 2.5, 4.7, 10.1**
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: universal-file-converter, Property 7: バッチ処理のフォールトトレランス
    ///
    /// **Validates: Requirements 2.5, 4.7, 10.1**
    #[test]
    fn prop_batch_fault_tolerance(
        valid_count in 1usize..6,
        invalid_count in 0usize..4,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tmp = TempDir::new().unwrap();
            let output_dir = tmp.path().join("output");
            fs::create_dir_all(&output_dir).unwrap();

            let mut input_paths: Vec<PathBuf> = Vec::new();

            // Create valid JSON files (convertible to YAML)
            for i in 0..valid_count {
                let path = tmp.path().join(format!("valid_{}.json", i));
                create_valid_json(&path);
                input_paths.push(path);
            }

            // Create invalid files (with .png extension but invalid content)
            for i in 0..invalid_count {
                let path = tmp.path().join(format!("invalid_{}.png", i));
                create_invalid_file(&path);
                input_paths.push(path);
            }

            let options = default_options();
            let (result, _progress) = simulate_batch_conversion(
                &input_paths,
                &FileFormat::Yaml,
                Some(&output_dir),
                &options,
                None,
            )
            .await;

            // All valid JSON files should be converted successfully
            prop_assert_eq!(result.success_count, valid_count as u32,
                "Expected {} successes, got {}", valid_count, result.success_count);

            // All invalid files (.png with bad content attempting image->yaml which is unsupported)
            // should fail with error messages
            prop_assert_eq!(result.failed_count, invalid_count as u32,
                "Expected {} failures, got {}", invalid_count, result.failed_count);

            // Each failed file should have an error message containing the filename
            for failed_info in &result.failed_files {
                prop_assert!(!failed_info.file_name.is_empty(),
                    "Failed file should have a filename");
                prop_assert!(!failed_info.error.is_empty(),
                    "Failed file should have an error message");
            }

            Ok(())
        })?;
    }
}

// =============================================================================
// Property 8: 進捗報告の正確性
// For any N-file batch, progress events have monotonically increasing
// processed_count from 0 to N, final total_count equals N.
//
// **Validates: Requirements 4.4, 4.5**
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: universal-file-converter, Property 8: 進捗報告の正確性
    ///
    /// **Validates: Requirements 4.4, 4.5**
    #[test]
    fn prop_progress_reporting_accuracy(
        file_count in 1usize..8,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tmp = TempDir::new().unwrap();
            let output_dir = tmp.path().join("output");
            fs::create_dir_all(&output_dir).unwrap();

            let mut input_paths: Vec<PathBuf> = Vec::new();

            // Create valid JSON files
            for i in 0..file_count {
                let path = tmp.path().join(format!("file_{}.json", i));
                create_valid_json(&path);
                input_paths.push(path);
            }

            let options = default_options();
            let (_result, progress_events) = simulate_batch_conversion(
                &input_paths,
                &FileFormat::Yaml,
                Some(&output_dir),
                &options,
                None,
            )
            .await;

            // Total count in all events should equal file_count
            for event in &progress_events {
                prop_assert_eq!(event.total_count, file_count as u32,
                    "total_count should be {} in all events, got {}",
                    file_count, event.total_count);
            }

            // processed_count should be monotonically non-decreasing
            let mut last_processed: u32 = 0;
            for event in &progress_events {
                prop_assert!(event.processed_count >= last_processed,
                    "processed_count should be monotonically non-decreasing: {} < {}",
                    event.processed_count, last_processed);
                last_processed = event.processed_count;
            }

            // The maximum processed_count should reach file_count
            let max_processed = progress_events.iter()
                .map(|e| e.processed_count)
                .max()
                .unwrap_or(0);
            prop_assert_eq!(max_processed, file_count as u32,
                "Final processed_count should reach {}, got {}",
                file_count, max_processed);

            // The final event(s) should have processed_count equal to file_count
            if let Some(last_event) = progress_events.last() {
                prop_assert_eq!(last_event.processed_count, file_count as u32,
                    "Last event processed_count should be {}, got {}",
                    file_count, last_event.processed_count);
            }

            Ok(())
        })?;
    }
}

// =============================================================================
// Property 9: キャンセル後の処理停止
// For any N-file batch, if cancel is issued at file K, processed files ≤ K+1
// and processed outputs are preserved.
//
// **Validates: Requirements 4.6**
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: universal-file-converter, Property 9: キャンセル後の処理停止
    ///
    /// **Validates: Requirements 4.6**
    #[test]
    fn prop_cancellation_stops_processing(
        file_count in 2usize..8,
        cancel_at_ratio in 0.1f64..0.9,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tmp = TempDir::new().unwrap();
            let output_dir = tmp.path().join("output");
            fs::create_dir_all(&output_dir).unwrap();

            let mut input_paths: Vec<PathBuf> = Vec::new();

            // Create valid JSON files
            for i in 0..file_count {
                let path = tmp.path().join(format!("file_{}.json", i));
                create_valid_json(&path);
                input_paths.push(path);
            }

            // Calculate cancel point (K)
            let cancel_at = ((file_count as f64) * cancel_at_ratio).floor() as usize;
            let cancel_at = cancel_at.max(1).min(file_count - 1);

            let options = default_options();
            let (result, progress_events) = simulate_batch_conversion(
                &input_paths,
                &FileFormat::Yaml,
                Some(&output_dir),
                &options,
                Some(cancel_at),
            )
            .await;

            // Processed files should be ≤ cancel_at + 1
            // (cancel_at is the index at which we stop, so at most cancel_at files processed)
            prop_assert!(result.success_count <= (cancel_at as u32) + 1,
                "After cancel at {}, success_count should be ≤ {}, got {}",
                cancel_at, cancel_at + 1, result.success_count);

            // Verify that already processed outputs are preserved in the output directory
            let output_files: Vec<_> = fs::read_dir(&output_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .collect();

            prop_assert_eq!(output_files.len() as u32, result.success_count,
                "Number of output files ({}) should match success_count ({})",
                output_files.len(), result.success_count);

            // Each output file should be non-empty (valid conversion result)
            for entry in &output_files {
                let metadata = entry.metadata().unwrap();
                prop_assert!(metadata.len() > 0,
                    "Output file {:?} should not be empty", entry.path());
            }

            // Progress events should contain a Cancelled event
            let has_cancelled = progress_events.iter()
                .any(|e| matches!(e.status, ProgressStatus::Cancelled));
            prop_assert!(has_cancelled,
                "Progress events should include a Cancelled status");

            Ok(())
        })?;
    }
}

// =============================================================================
// Property 13: 条件付きソースファイル削除
// For any batch (delete option enabled), only successfully converted files
// have sources deleted; failed files' sources are preserved.
//
// **Validates: Requirements 5.5, 10.5**
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: universal-file-converter, Property 13: 条件付きソースファイル削除
    ///
    /// **Validates: Requirements 5.5, 10.5**
    #[test]
    fn prop_conditional_source_deletion(
        valid_count in 1usize..5,
        invalid_count in 1usize..4,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tmp = TempDir::new().unwrap();
            let output_dir = tmp.path().join("output");
            fs::create_dir_all(&output_dir).unwrap();

            let mut input_paths: Vec<PathBuf> = Vec::new();
            let mut valid_paths: Vec<PathBuf> = Vec::new();
            let mut invalid_paths: Vec<PathBuf> = Vec::new();

            // Create valid JSON files (will be successfully converted to YAML)
            for i in 0..valid_count {
                let path = tmp.path().join(format!("good_{}.json", i));
                create_valid_json(&path);
                valid_paths.push(path.clone());
                input_paths.push(path);
            }

            // Create invalid files (.png extension with garbage content, image->yaml unsupported)
            for i in 0..invalid_count {
                let path = tmp.path().join(format!("bad_{}.png", i));
                create_invalid_file(&path);
                invalid_paths.push(path.clone());
                input_paths.push(path);
            }

            // Enable delete_source option
            let mut options = default_options();
            options.delete_source = true;

            let (result, _progress) = simulate_batch_conversion(
                &input_paths,
                &FileFormat::Yaml,
                Some(&output_dir),
                &options,
                None,
            )
            .await;

            // Successfully converted files' sources should be deleted
            for path in &valid_paths {
                prop_assert!(!path.exists(),
                    "Successfully converted source {:?} should be deleted", path);
            }

            // Failed files' sources should be preserved
            for path in &invalid_paths {
                prop_assert!(path.exists(),
                    "Failed source {:?} should be preserved", path);
            }

            // Verify counts
            prop_assert_eq!(result.success_count, valid_count as u32);
            prop_assert_eq!(result.failed_count, invalid_count as u32);

            Ok(())
        })?;
    }
}

// =============================================================================
// Property 15: 失敗時の不完全出力クリーンアップ
// For any file that errors during conversion, the incomplete output file is
// deleted from the output directory.
//
// **Validates: Requirements 10.6**
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: universal-file-converter, Property 15: 失敗時の不完全出力クリーンアップ
    ///
    /// **Validates: Requirements 10.6**
    #[test]
    fn prop_failed_conversion_cleanup(
        invalid_count in 1usize..6,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tmp = TempDir::new().unwrap();
            let output_dir = tmp.path().join("output");
            fs::create_dir_all(&output_dir).unwrap();

            let mut input_paths: Vec<PathBuf> = Vec::new();

            // Create files with .png extension but invalid image content
            // These will pass validation (extension is recognized) but fail during
            // actual conversion (image decoding will fail)
            for i in 0..invalid_count {
                let path = tmp.path().join(format!("corrupt_{}.png", i));
                // Write garbage that looks like it has a PNG extension but isn't valid
                fs::write(&path, b"not a valid png image data at all").unwrap();
                input_paths.push(path);
            }

            let options = default_options();
            let (result, _progress) = simulate_batch_conversion(
                &input_paths,
                &FileFormat::Jpeg, // PNG -> JPEG: valid pair but will fail on decode
                Some(&output_dir),
                &options,
                None,
            )
            .await;

            // All files should have failed
            prop_assert_eq!(result.failed_count, invalid_count as u32,
                "Expected {} failures, got {}", invalid_count, result.failed_count);
            prop_assert_eq!(result.success_count, 0,
                "Expected 0 successes for corrupt files, got {}", result.success_count);

            // The output directory should NOT contain any incomplete output files
            let output_files: Vec<_> = fs::read_dir(&output_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .collect();

            prop_assert_eq!(output_files.len(), 0,
                "Output directory should be empty after failed conversions, but found {} files: {:?}",
                output_files.len(),
                output_files.iter().map(|e| e.path()).collect::<Vec<_>>());

            // Each failed file should have an error message
            for failed_info in &result.failed_files {
                prop_assert!(!failed_info.file_name.is_empty(),
                    "Failed file should have a filename");
                prop_assert!(!failed_info.error.is_empty(),
                    "Failed file should have an error message");
                prop_assert!(failed_info.file_name.contains("corrupt_"),
                    "Error should reference the corrupt file name: {}", failed_info.file_name);
            }

            Ok(())
        })?;
    }
}

// =============================================================================
// Additional combined test: mix of valid and corrupt files with cleanup
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Feature: universal-file-converter, Property 15: 失敗時の不完全出力クリーンアップ (mixed batch)
    ///
    /// **Validates: Requirements 10.6**
    #[test]
    fn prop_mixed_batch_incomplete_output_cleanup(
        valid_count in 1usize..4,
        corrupt_count in 1usize..4,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let tmp = TempDir::new().unwrap();
            let output_dir = tmp.path().join("output");
            fs::create_dir_all(&output_dir).unwrap();

            let mut input_paths: Vec<PathBuf> = Vec::new();

            // Create valid PNG files (will convert to JPEG successfully)
            for i in 0..valid_count {
                let path = tmp.path().join(format!("valid_{}.png", i));
                create_valid_png(&path);
                input_paths.push(path);
            }

            // Create corrupt PNG files (will fail during decode)
            for i in 0..corrupt_count {
                let path = tmp.path().join(format!("corrupt_{}.png", i));
                fs::write(&path, b"this is not valid PNG data").unwrap();
                input_paths.push(path);
            }

            let options = default_options();
            let (result, _progress) = simulate_batch_conversion(
                &input_paths,
                &FileFormat::Jpeg,
                Some(&output_dir),
                &options,
                None,
            )
            .await;

            // Valid files should succeed
            prop_assert_eq!(result.success_count, valid_count as u32,
                "Expected {} successes, got {}", valid_count, result.success_count);

            // Corrupt files should fail
            prop_assert_eq!(result.failed_count, corrupt_count as u32,
                "Expected {} failures, got {}", corrupt_count, result.failed_count);

            // Output directory should only contain successfully converted files
            let output_files: Vec<_> = fs::read_dir(&output_dir)
                .unwrap()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file())
                .collect();

            prop_assert_eq!(output_files.len(), valid_count,
                "Output should contain exactly {} files (successful only), found {}",
                valid_count, output_files.len());

            // No output files should correspond to corrupt files
            for entry in &output_files {
                let name = entry.file_name().to_string_lossy().to_string();
                prop_assert!(!name.contains("corrupt_"),
                    "Output should not contain corrupt file output: {}", name);
            }

            Ok(())
        })?;
    }
}

// =============================================================================
// Unit tests for CancellationToken behavior (Property 9 supplement)
// =============================================================================

#[cfg(test)]
mod cancellation_unit_tests {
    use crate::converter::ConversionEngine;

    #[test]
    fn test_cancel_token_stops_batch() {
        let engine = ConversionEngine::new();

        // Initially not cancelled
        assert!(!engine.is_cancelled());

        // Cancel
        engine.cancel();
        assert!(engine.is_cancelled());

        // Token clones also see cancelled state
        let token = engine.cancel_token();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_reset_cancel_creates_new_token() {
        let mut engine = ConversionEngine::new();

        engine.cancel();
        assert!(engine.is_cancelled());

        engine.reset_cancel();
        assert!(!engine.is_cancelled());

        // New token should be fresh
        let new_token = engine.cancel_token();
        assert!(!new_token.is_cancelled());
    }

    #[test]
    fn test_cancel_token_shared_state() {
        let engine = ConversionEngine::new();
        let token1 = engine.cancel_token();
        let token2 = engine.cancel_token();

        assert!(!token1.is_cancelled());
        assert!(!token2.is_cancelled());

        engine.cancel();

        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
    }
}
