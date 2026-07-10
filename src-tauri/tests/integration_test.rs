//! Integration tests for Universal File Converter
//!
//! Tests the end-to-end conversion flow, file system operations,
//! history persistence, and batch processing without requiring the Tauri runtime.
//!
//! Validates: Requirements 1.4, 8.1, 5.2

use std::fs;
use std::path::{Path, PathBuf};

use tempfile::TempDir;

use webpexer_lib::converter::image::ImageConverter;
use webpexer_lib::converter::text::TextConverter;
use webpexer_lib::converter::{ConversionEngine, Converter, ConverterBoxed};
use webpexer_lib::history::{HistoryEntry, HistoryState, HistoryStatus, HistoryStore};
use webpexer_lib::models::{
    ConflictResolution, ConversionOptions, ConversionResult,
    FailedFileInfo, FileFormat, FileNamingPattern,
};
use webpexer_lib::output::{ensure_output_dir, generate_output_path, OutputPathResult};
use webpexer_lib::validator::Validator;

use chrono::Utc;

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

/// Create a valid PNG image at the specified path
fn create_valid_png(path: &Path) {
    let img = image::DynamicImage::new_rgba8(4, 4);
    img.save(path).expect("Failed to create test PNG image");
}

/// Create a valid JSON file at the specified path
fn create_valid_json(path: &Path) {
    fs::write(path, r#"{"key": "value", "number": 42}"#)
        .expect("Failed to create test JSON");
}

/// Create an invalid file (not a valid image)
fn create_invalid_file(path: &Path) {
    fs::write(path, b"this is not valid image data at all!!!")
        .expect("Failed to create invalid file");
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

/// Simulates the batch conversion loop from start_conversion.
/// This replicates the core logic without requiring Tauri State or Channel.
async fn simulate_batch_conversion(
    input_paths: &[PathBuf],
    output_format: &FileFormat,
    output_dir: Option<&Path>,
    options: &ConversionOptions,
) -> ConversionResult {
    let mut success_count: u32 = 0;
    let mut failed_count: u32 = 0;
    let mut skipped_count: u32 = 0;
    let mut failed_files: Vec<FailedFileInfo> = Vec::new();

    let validation = Validator::validate_input_files(input_paths);
    let valid_files = validation.valid_files;

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

    for (idx, (file_path, input_format)) in valid_files.iter().enumerate() {
        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

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
                error: error_msg,
            });
            failed_count += 1;
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
                    error: error_msg,
                });
                failed_count += 1;
            }
        }
    }

    ConversionResult {
        success_count,
        failed_count,
        skipped_count,
        failed_files,
        output_dir: result_output_dir,
    }
}

// =============================================================================
// 1. End-to-end conversion flow (without Tauri)
// =============================================================================

/// Test: PNG → JPEG conversion produces a decodable JPEG file
#[tokio::test]
async fn test_e2e_image_conversion_png_to_jpeg() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("photo.png");
    let output = tmp.path().join("photo.jpeg");
    create_valid_png(&input);

    let converter = ImageConverter::new();
    let opts = default_options();
    converter.convert(&input, &output, &opts).await.unwrap();

    // Verify output exists and is a valid JPEG
    assert!(output.exists());
    assert!(output.metadata().unwrap().len() > 0);
    let decoded = image::open(&output);
    assert!(decoded.is_ok(), "Output JPEG is not decodable");
}

/// Test: JSON → YAML conversion produces valid YAML
#[tokio::test]
async fn test_e2e_text_conversion_json_to_yaml() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("data.json");
    let output = tmp.path().join("data.yaml");

    let json_content = r#"{"name": "test", "count": 5, "active": true}"#;
    fs::write(&input, json_content).unwrap();

    let converter = TextConverter::new();
    let opts = default_options();
    converter.convert(&input, &output, &opts).await.unwrap();

    // Verify output exists and is parseable YAML
    assert!(output.exists());
    let yaml_content = fs::read_to_string(&output).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&yaml_content).unwrap();
    assert_eq!(parsed["name"], serde_yaml::Value::String("test".to_string()));
}

/// Test: CSV → JSON conversion preserves data integrity
#[tokio::test]
async fn test_e2e_text_conversion_csv_to_json() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("data.csv");
    let output = tmp.path().join("data.json");

    fs::write(&input, "id,name,score\n1,Alice,95\n2,Bob,87\n").unwrap();

    let converter = TextConverter::new();
    let opts = default_options();
    converter.convert(&input, &output, &opts).await.unwrap();

    // Verify output is valid JSON array
    assert!(output.exists());
    let json_content = fs::read_to_string(&output).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&json_content).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["name"], "Alice");
    assert_eq!(arr[1]["name"], "Bob");
}

/// Test: JSON → YAML → JSON roundtrip preserves data
#[tokio::test]
async fn test_e2e_roundtrip_json_yaml_json() {
    let tmp = TempDir::new().unwrap();
    let json_input = tmp.path().join("original.json");
    let yaml_intermediate = tmp.path().join("intermediate.yaml");
    let json_output = tmp.path().join("roundtrip.json");

    let original = r#"{"name": "roundtrip", "values": [1, 2, 3], "nested": {"a": true}}"#;
    fs::write(&json_input, original).unwrap();

    let converter = TextConverter::new();
    let opts = default_options();

    // JSON → YAML
    converter.convert(&json_input, &yaml_intermediate, &opts).await.unwrap();
    assert!(yaml_intermediate.exists());

    // YAML → JSON
    converter.convert(&yaml_intermediate, &json_output, &opts).await.unwrap();
    assert!(json_output.exists());

    // Compare original and roundtrip
    let original_val: serde_json::Value = serde_json::from_str(original).unwrap();
    let roundtrip_content = fs::read_to_string(&json_output).unwrap();
    let roundtrip_val: serde_json::Value = serde_json::from_str(&roundtrip_content).unwrap();
    assert_eq!(original_val, roundtrip_val);
}

/// Test: Image conversion with resize option
#[tokio::test]
async fn test_e2e_image_conversion_with_resize() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("large.png");
    let output = tmp.path().join("resized.png");

    // Create a 100x50 image
    let img = image::DynamicImage::new_rgba8(100, 50);
    img.save(&input).unwrap();

    let mut opts = default_options();
    opts.resize = Some(webpexer_lib::models::ResizeOptions {
        width: Some(50),
        height: None,
        maintain_aspect_ratio: true,
    });

    let converter = ImageConverter::new();
    converter.convert(&input, &output, &opts).await.unwrap();

    // Verify dimensions: width=50, height=round(50*50/100)=25
    let decoded = image::open(&output).unwrap();
    assert_eq!(decoded.width(), 50);
    assert_eq!(decoded.height(), 25);
}

// =============================================================================
// 2. File system operations
// =============================================================================

/// Test: ensure_output_dir creates the output directory
#[test]
fn test_fs_ensure_output_dir_creates_directory() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("image.png");
    fs::write(&source, b"data").unwrap();

    // With no explicit output_dir, it creates {source_dir}/{format_name}/
    let result = ensure_output_dir(&source, None, &FileFormat::Jpeg);
    assert!(result.is_ok());
    let dir = result.unwrap();
    assert!(dir.exists());
    assert_eq!(dir, tmp.path().join("jpeg"));
}

/// Test: File writing and reading back
#[tokio::test]
async fn test_fs_write_and_read_back() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("source.json");
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();
    let output = output_dir.join("source.yaml");

    let content = r#"{"hello": "world", "num": 123}"#;
    fs::write(&input, content).unwrap();

    let converter = TextConverter::new();
    let opts = default_options();
    converter.convert(&input, &output, &opts).await.unwrap();

    // Read back and verify content
    let written = fs::read_to_string(&output).unwrap();
    assert!(written.contains("hello"));
    assert!(written.contains("world"));

    // Verify it's valid YAML
    let parsed: serde_yaml::Value = serde_yaml::from_str(&written).unwrap();
    assert_eq!(parsed["num"], serde_yaml::Value::Number(serde_yaml::Number::from(123)));
}

/// Test: Source file deletion when delete_source is enabled
#[tokio::test]
async fn test_fs_source_deletion_on_success() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("to_delete.json");
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    create_valid_json(&input);
    assert!(input.exists());

    let mut opts = default_options();
    opts.delete_source = true;

    let input_paths = vec![input.clone()];
    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Yaml,
        Some(&output_dir),
        &opts,
    )
    .await;

    assert_eq!(result.success_count, 1);
    // Source should be deleted after successful conversion
    assert!(!input.exists(), "Source file should be deleted");
}

/// Test: Incomplete output cleanup on conversion failure
#[tokio::test]
async fn test_fs_cleanup_incomplete_output_on_failure() {
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    // Create a corrupt PNG file (valid extension, invalid content)
    let input = tmp.path().join("corrupt.png");
    create_invalid_file(&input);

    let input_paths = vec![input.clone()];
    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Jpeg,
        Some(&output_dir),
        &default_options(),
    )
    .await;

    assert_eq!(result.failed_count, 1);
    assert_eq!(result.success_count, 0);

    // No incomplete output files should remain
    let output_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    assert_eq!(
        output_files.len(),
        0,
        "No incomplete output files should remain after failure"
    );
}

/// Test: Custom output directory creation
#[test]
fn test_fs_custom_output_dir_creation() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("file.png");
    fs::write(&source, b"data").unwrap();

    let custom_dir = tmp.path().join("deep").join("nested").join("output");
    assert!(!custom_dir.exists());

    let result = ensure_output_dir(&source, Some(&custom_dir), &FileFormat::Png);
    assert!(result.is_ok());
    assert!(custom_dir.exists());
}

// =============================================================================
// 3. History persistence
// =============================================================================

/// Helper: create a test history entry
fn make_history_entry(id: &str) -> HistoryEntry {
    HistoryEntry {
        id: id.to_string(),
        source_folder: "/tmp/test".to_string(),
        output_format: FileFormat::Png,
        cleanup_enabled: false,
        executed_at: Utc::now(),
        status: HistoryStatus::Success,
        file_count: 5,
    }
}

/// Test: HistoryStore persists entries to file and reloads them
#[test]
fn test_history_persistence_save_and_reload() {
    let tmp = TempDir::new().unwrap();
    let storage_path = tmp.path().join("history.json");

    // Create store, add entries
    {
        let mut store = HistoryStore::with_storage_path(storage_path.clone());
        store.add_entry(make_history_entry("entry-1"));
        store.add_entry(make_history_entry("entry-2"));
        store.add_entry(make_history_entry("entry-3"));
    }

    // File should exist
    assert!(storage_path.exists());

    // Reload from file and verify
    let store = HistoryStore::with_storage_path(storage_path);
    assert_eq!(store.get_all().len(), 3);
    // Most recent first
    assert_eq!(store.get_all()[0].id, "entry-3");
    assert_eq!(store.get_all()[1].id, "entry-2");
    assert_eq!(store.get_all()[2].id, "entry-1");
}

/// Test: HistoryStore clear removes entries and updates file
#[test]
fn test_history_persistence_clear_updates_file() {
    let tmp = TempDir::new().unwrap();
    let storage_path = tmp.path().join("history.json");

    // Create and populate
    {
        let mut store = HistoryStore::with_storage_path(storage_path.clone());
        store.add_entry(make_history_entry("a"));
        store.add_entry(make_history_entry("b"));
    }

    // Clear
    {
        let mut store = HistoryStore::with_storage_path(storage_path.clone());
        assert_eq!(store.get_all().len(), 2);
        store.clear();
    }

    // Reload should be empty
    let store = HistoryStore::with_storage_path(storage_path);
    assert_eq!(store.get_all().len(), 0);
}

/// Test: HistoryStore respects max_entries limit (50)
#[test]
fn test_history_persistence_max_entries_limit() {
    let tmp = TempDir::new().unwrap();
    let storage_path = tmp.path().join("history.json");

    let mut store = HistoryStore::with_storage_path(storage_path.clone());

    // Add 55 entries
    for i in 0..55 {
        store.add_entry(make_history_entry(&format!("entry-{}", i)));
    }

    // Should be capped at 50
    assert_eq!(store.get_all().len(), 50);
    // Most recent should be first
    assert_eq!(store.get_all()[0].id, "entry-54");
    // Oldest surviving entry
    assert_eq!(store.get_all()[49].id, "entry-5");

    // Reload and verify persistence
    let reloaded = HistoryStore::with_storage_path(storage_path);
    assert_eq!(reloaded.get_all().len(), 50);
    assert_eq!(reloaded.get_all()[0].id, "entry-54");
}

/// Test: HistoryState manages persistence via app_data_dir
#[test]
fn test_history_state_persistence() {
    let tmp = TempDir::new().unwrap();
    let state = HistoryState::new(tmp.path());

    // Add entries via state
    {
        let mut store = state.store.lock().unwrap();
        store.add_entry(make_history_entry("state-1"));
        store.add_entry(make_history_entry("state-2"));
    }

    // Verify history.json was created
    let history_file = tmp.path().join("history.json");
    assert!(history_file.exists());

    // Create new state from same directory - should load entries
    let state2 = HistoryState::new(tmp.path());
    let store = state2.store.lock().unwrap();
    assert_eq!(store.get_all().len(), 2);
    assert_eq!(store.get_all()[0].id, "state-2");
}

/// Test: History data integrity after serialization/deserialization
#[test]
fn test_history_data_integrity() {
    let tmp = TempDir::new().unwrap();
    let storage_path = tmp.path().join("history.json");

    let entry = HistoryEntry {
        id: "integrity-test".to_string(),
        source_folder: "/path/to/folder".to_string(),
        output_format: FileFormat::Jpeg,
        cleanup_enabled: true,
        executed_at: Utc::now(),
        status: HistoryStatus::Failed,
        file_count: 42,
    };

    {
        let mut store = HistoryStore::with_storage_path(storage_path.clone());
        store.add_entry(entry.clone());
    }

    // Reload and verify all fields
    let store = HistoryStore::with_storage_path(storage_path);
    let loaded = &store.get_all()[0];
    assert_eq!(loaded.id, "integrity-test");
    assert_eq!(loaded.source_folder, "/path/to/folder");
    assert_eq!(loaded.output_format, FileFormat::Jpeg);
    assert_eq!(loaded.cleanup_enabled, true);
    assert_eq!(loaded.file_count, 42);
    assert!(matches!(loaded.status, HistoryStatus::Failed));
}

// =============================================================================
// 4. Complete batch flow simulation
// =============================================================================

/// Test: Mixed batch with valid PNGs + valid JSONs + invalid files
#[tokio::test]
async fn test_batch_mixed_valid_and_invalid_files() {
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut input_paths: Vec<PathBuf> = Vec::new();

    // 2 valid PNG files (convert to JPEG)
    for i in 0..2 {
        let path = tmp.path().join(format!("valid_{}.png", i));
        create_valid_png(&path);
        input_paths.push(path);
    }

    // 1 corrupt PNG file (will fail on decode)
    let corrupt = tmp.path().join("corrupt.png");
    create_invalid_file(&corrupt);
    input_paths.push(corrupt);

    // 1 unsupported extension file
    let unsupported = tmp.path().join("file.xyz");
    fs::write(&unsupported, b"data").unwrap();
    input_paths.push(unsupported);

    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Jpeg,
        Some(&output_dir),
        &default_options(),
    )
    .await;

    // 2 valid PNGs should succeed
    assert_eq!(result.success_count, 2);
    // 1 corrupt PNG (decode error) + 1 unsupported format = 2 failures
    assert_eq!(result.failed_count, 2);
    assert_eq!(result.failed_files.len(), 2);

    // Output dir should only have 2 files (successful conversions)
    let output_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();
    assert_eq!(output_files.len(), 2);
}

/// Test: Batch with sequential naming pattern
#[tokio::test]
async fn test_batch_sequential_naming_pattern() {
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut input_paths: Vec<PathBuf> = Vec::new();
    for i in 0..3 {
        let path = tmp.path().join(format!("img_{}.png", i));
        create_valid_png(&path);
        input_paths.push(path);
    }

    let mut opts = default_options();
    opts.file_naming = FileNamingPattern::OriginalSequential;

    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Jpeg,
        Some(&output_dir),
        &opts,
    )
    .await;

    assert_eq!(result.success_count, 3);

    // Verify output files have sequential naming
    assert!(output_dir.join("img_0_001.jpeg").exists()
        || output_dir.join("img_0_001.jpg").exists());
    assert!(output_dir.join("img_1_002.jpeg").exists()
        || output_dir.join("img_1_002.jpg").exists());
    assert!(output_dir.join("img_2_003.jpeg").exists()
        || output_dir.join("img_2_003.jpg").exists());
}

/// Test: Batch with conflict resolution (skip mode)
#[tokio::test]
async fn test_batch_conflict_resolution_skip() {
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    // Pre-create an output file (simulating existing file)
    let existing_output = output_dir.join("file.yaml");
    fs::write(&existing_output, "existing: content").unwrap();

    let input = tmp.path().join("file.json");
    create_valid_json(&input);

    let mut opts = default_options();
    opts.conflict_resolution = ConflictResolution::Skip;

    let input_paths = vec![input.clone()];
    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Yaml,
        Some(&output_dir),
        &opts,
    )
    .await;

    // File should be skipped since output already exists
    assert_eq!(result.skipped_count, 1);
    assert_eq!(result.success_count, 0);

    // Original output file should be unchanged
    let content = fs::read_to_string(&existing_output).unwrap();
    assert_eq!(content, "existing: content");
}

/// Test: Batch with conflict resolution (rename mode)
#[tokio::test]
async fn test_batch_conflict_resolution_rename() {
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    // Pre-create an output file
    let existing_output = output_dir.join("file.yaml");
    fs::write(&existing_output, "existing: content").unwrap();

    let input = tmp.path().join("file.json");
    create_valid_json(&input);

    let mut opts = default_options();
    opts.conflict_resolution = ConflictResolution::Rename;

    let input_paths = vec![input.clone()];
    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Yaml,
        Some(&output_dir),
        &opts,
    )
    .await;

    // Should succeed with a renamed file
    assert_eq!(result.success_count, 1);

    // Original file should still have old content
    let original_content = fs::read_to_string(&existing_output).unwrap();
    assert_eq!(original_content, "existing: content");

    // A new file with _2 suffix should exist
    let renamed = output_dir.join("file_2.yaml");
    assert!(renamed.exists(), "Renamed output file should exist");
}

/// Test: Batch with delete_source only deletes successfully converted files
#[tokio::test]
async fn test_batch_delete_source_only_on_success() {
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let valid = tmp.path().join("good.json");
    create_valid_json(&valid);

    let corrupt = tmp.path().join("bad.png");
    create_invalid_file(&corrupt);

    let mut opts = default_options();
    opts.delete_source = true;

    let input_paths = vec![valid.clone(), corrupt.clone()];
    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Yaml,
        Some(&output_dir),
        &opts,
    )
    .await;

    assert_eq!(result.success_count, 1);
    assert_eq!(result.failed_count, 1);

    // Successful file's source should be deleted
    assert!(!valid.exists(), "Successfully converted source should be deleted");
    // Failed file's source should be preserved
    assert!(corrupt.exists(), "Failed source should be preserved");
}

/// Test: Batch text conversion (JSON files to TOML)
#[tokio::test]
async fn test_batch_text_json_to_toml() {
    let tmp = TempDir::new().unwrap();
    let output_dir = tmp.path().join("output");
    fs::create_dir_all(&output_dir).unwrap();

    let mut input_paths: Vec<PathBuf> = Vec::new();
    let contents = vec![
        r#"{"server": "localhost", "port": 8080}"#,
        r#"{"name": "app", "version": "1.0"}"#,
    ];

    for (i, content) in contents.iter().enumerate() {
        let path = tmp.path().join(format!("config_{}.json", i));
        fs::write(&path, content).unwrap();
        input_paths.push(path);
    }

    let result = simulate_batch_conversion(
        &input_paths,
        &FileFormat::Toml,
        Some(&output_dir),
        &default_options(),
    )
    .await;

    assert_eq!(result.success_count, 2);
    assert_eq!(result.failed_count, 0);

    // Verify outputs are valid TOML
    let output_files: Vec<_> = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|e| e == "toml").unwrap_or(false))
        .collect();
    assert_eq!(output_files.len(), 2);

    for entry in &output_files {
        let content = fs::read_to_string(entry.path()).unwrap();
        let parsed: toml::Value = toml::from_str(&content).unwrap();
        assert!(parsed.is_table());
    }
}

/// Test: ConversionEngine correctly selects converter for format pairs
#[test]
fn test_engine_converter_selection() {
    let engine = ConversionEngine::new();

    // Image pairs
    assert!(engine.can_convert(&FileFormat::Png, &FileFormat::Jpeg));
    assert!(engine.can_convert(&FileFormat::Webp, &FileFormat::Bmp));
    assert!(engine.can_convert(&FileFormat::Gif, &FileFormat::Tiff));

    // Text pairs
    assert!(engine.can_convert(&FileFormat::Json, &FileFormat::Yaml));
    assert!(engine.can_convert(&FileFormat::Csv, &FileFormat::Json));
    assert!(engine.can_convert(&FileFormat::Toml, &FileFormat::Xml));

    // Cross-category should NOT work
    assert!(!engine.can_convert(&FileFormat::Png, &FileFormat::Json));
    assert!(!engine.can_convert(&FileFormat::Json, &FileFormat::Png));

    // Same format should NOT work
    assert!(!engine.can_convert(&FileFormat::Png, &FileFormat::Png));
    assert!(!engine.can_convert(&FileFormat::Json, &FileFormat::Json));
}
