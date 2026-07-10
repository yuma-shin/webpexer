use std::fs;

use proptest::prelude::*;
use tempfile::TempDir;

use crate::models::{ConflictResolution, FileFormat, FileNamingPattern};
use crate::output::{
    format_extension, format_name, generate_output_path, OutputPathResult,
};

/// FileFormat の任意値を生成する Strategy
fn arb_file_format() -> impl Strategy<Value = FileFormat> {
    prop_oneof![
        Just(FileFormat::Png),
        Just(FileFormat::Jpeg),
        Just(FileFormat::Webp),
        Just(FileFormat::Gif),
        Just(FileFormat::Bmp),
        Just(FileFormat::Tiff),
        Just(FileFormat::Avif),
        Just(FileFormat::Ico),
        Just(FileFormat::Json),
        Just(FileFormat::Yaml),
        Just(FileFormat::Toml),
        Just(FileFormat::Xml),
        Just(FileFormat::Csv),
        Just(FileFormat::Markdown),
        Just(FileFormat::PlainText),
    ]
}

/// 有効なファイル名ステム（拡張子なし）を生成する Strategy
/// ファイルシステムで安全な ASCII 文字のみ使用
fn arb_filename_stem() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9_-]{0,19}".prop_filter("non-empty stem", |s| !s.is_empty())
}

/// 有効な拡張子を生成する Strategy
fn arb_extension() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("png".to_string()),
        Just("jpg".to_string()),
        Just("webp".to_string()),
        Just("gif".to_string()),
        Just("bmp".to_string()),
        Just("tiff".to_string()),
        Just("json".to_string()),
        Just("yaml".to_string()),
        Just("toml".to_string()),
        Just("xml".to_string()),
        Just("csv".to_string()),
        Just("md".to_string()),
        Just("txt".to_string()),
    ]
}

// =============================================================================
// Property 10: デフォルト出力パス生成
// Feature: universal-file-converter, Property 10: デフォルト出力パス生成
// Validates: Requirements 5.2
//
// For any source file path and output format name, the default output path
// (when no output dir is specified) matches the pattern
// `{source_dir}/{format_name}/{original_filename}.{format_ext}`.
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 5.2**
    #[test]
    fn prop_default_output_path_pattern(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
    ) {
        let tmp = TempDir::new().unwrap();
        let source_filename = format!("{}.{}", stem, src_ext);
        let source = tmp.path().join(&source_filename);
        fs::write(&source, b"test data").unwrap();

        let result = generate_output_path(
            &source,
            None, // no output dir specified
            &output_format,
            &FileNamingPattern::Original,
            &ConflictResolution::Overwrite,
            1,
        );

        // Expected pattern: {source_dir}/{format_name}/{original_stem}.{format_ext}
        let expected_dir = tmp.path().join(format_name(&output_format));
        let expected_filename = format!("{}.{}", stem, format_extension(&output_format));
        let expected_path = expected_dir.join(&expected_filename);

        prop_assert_eq!(
            result,
            OutputPathResult::Write(expected_path.clone()),
            "Default output path should be {{source_dir}}/{{format_name}}/{{original_stem}}.{{format_ext}}\n\
             source: {:?}\n\
             expected: {:?}",
            source,
            expected_path
        );
    }
}

// =============================================================================
// Property 11: ファイル命名パターンの正確性
// Feature: universal-file-converter, Property 11: ファイル命名パターンの正確性
// Validates: Requirements 5.3
//
// For any original filename and naming pattern:
// - `original` produces the original filename unchanged
// - `original_sequential` produces `{original_stem}_{NNN}` (zero-padded 3 digits)
// - `original_datetime` produces `{original_stem}_{yyyyMMdd_HHmmss}`
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_naming_pattern_original(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::Original,
            &ConflictResolution::Overwrite,
            1,
        );

        // `original` naming: filename is unchanged stem + new extension
        let expected_filename = format!("{}.{}", stem, format_extension(&output_format));
        let expected_path = output_dir.join(&expected_filename);

        prop_assert_eq!(
            result,
            OutputPathResult::Write(expected_path),
            "Original naming pattern should produce unchanged stem"
        );
    }

    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_naming_pattern_sequential(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
        seq_num in 0u32..1000,
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::OriginalSequential,
            &ConflictResolution::Overwrite,
            seq_num,
        );

        // `original_sequential` naming: {stem}_{NNN}.{ext}
        let expected_filename = format!(
            "{}_{:03}.{}",
            stem,
            seq_num,
            format_extension(&output_format)
        );
        let expected_path = output_dir.join(&expected_filename);

        prop_assert_eq!(
            result,
            OutputPathResult::Write(expected_path),
            "Sequential naming pattern should produce {{stem}}_{{NNN}} with zero-padded 3 digits"
        );
    }

    /// **Validates: Requirements 5.3**
    #[test]
    fn prop_naming_pattern_datetime(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::OriginalDatetime,
            &ConflictResolution::Overwrite,
            1,
        );

        match result {
            OutputPathResult::Write(path) => {
                let filename = path.file_name().unwrap().to_str().unwrap();
                let ext = format_extension(&output_format);

                // Should end with the correct extension
                prop_assert!(
                    filename.ends_with(&format!(".{}", ext)),
                    "Datetime naming should produce file with correct extension, got: {}",
                    filename
                );

                // Should start with the original stem followed by underscore
                prop_assert!(
                    filename.starts_with(&format!("{}_", stem)),
                    "Datetime naming should start with '{{stem}}_', got: {}",
                    filename
                );

                // Extract the datetime part: between "{stem}_" and ".{ext}"
                let prefix = format!("{}_", stem);
                let suffix = format!(".{}", ext);
                let datetime_part = &filename[prefix.len()..filename.len() - suffix.len()];

                // Datetime should be in yyyyMMdd_HHmmss format (15 chars)
                prop_assert_eq!(
                    datetime_part.len(),
                    15,
                    "Datetime part should be 15 chars (yyyyMMdd_HHmmss), got: '{}' (len={})",
                    datetime_part,
                    datetime_part.len()
                );

                // Validate format: 8 digits + underscore + 6 digits
                let parts: Vec<&str> = datetime_part.splitn(2, '_').collect();
                prop_assert_eq!(
                    parts.len(),
                    2,
                    "Datetime part should have exactly one underscore separator"
                );
                prop_assert_eq!(parts[0].len(), 8, "Date part should be 8 digits (yyyyMMdd)");
                prop_assert_eq!(parts[1].len(), 6, "Time part should be 6 digits (HHmmss)");
                prop_assert!(
                    parts[0].chars().all(|c| c.is_ascii_digit()),
                    "Date part should be all digits, got: {}",
                    parts[0]
                );
                prop_assert!(
                    parts[1].chars().all(|c| c.is_ascii_digit()),
                    "Time part should be all digits, got: {}",
                    parts[1]
                );
            }
            OutputPathResult::Skip => {
                prop_assert!(false, "Datetime naming with Overwrite should never Skip");
            }
        }
    }
}

// =============================================================================
// Property 12: ファイル衝突解決の正確性
// Feature: universal-file-converter, Property 12: ファイル衝突解決の正確性
// Validates: Requirements 5.4
//
// For any existing same-name file at the output destination:
// - `overwrite` overwrites the existing file
// - `skip` does not output and preserves the existing file
// - `rename` outputs with a `_N` (N≥2) suffix alternate name
// =============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_conflict_overwrite_returns_same_path(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"source data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        // Create an existing file at the output location
        let existing_filename = format!("{}.{}", stem, format_extension(&output_format));
        let existing_path = output_dir.join(&existing_filename);
        fs::write(&existing_path, b"existing content").unwrap();

        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::Original,
            &ConflictResolution::Overwrite,
            1,
        );

        // Overwrite should return the same path (the existing file path)
        prop_assert_eq!(
            result,
            OutputPathResult::Write(existing_path),
            "Overwrite conflict resolution should return the existing file path for writing"
        );
    }

    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_conflict_skip_preserves_existing(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"source data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        // Create an existing file at the output location
        let existing_filename = format!("{}.{}", stem, format_extension(&output_format));
        let existing_path = output_dir.join(&existing_filename);
        let existing_content = b"existing content should be preserved";
        fs::write(&existing_path, existing_content).unwrap();

        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::Original,
            &ConflictResolution::Skip,
            1,
        );

        // Skip should return Skip result
        prop_assert_eq!(
            result,
            OutputPathResult::Skip,
            "Skip conflict resolution should return Skip when file exists"
        );

        // Verify the existing file is untouched
        let content = fs::read(&existing_path).unwrap();
        prop_assert_eq!(
            content,
            existing_content.to_vec(),
            "Existing file content should be preserved after Skip"
        );
    }

    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_conflict_rename_uses_suffix(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
        extra_files in 0u32..5,
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"source data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        let ext = format_extension(&output_format);

        // Create the original file
        let original_filename = format!("{}.{}", stem, ext);
        let original_path = output_dir.join(&original_filename);
        fs::write(&original_path, b"original").unwrap();

        // Create sequential _N files (file_2, file_3, ... up to extra_files+1)
        for n in 2..=(extra_files + 1) {
            let extra_filename = format!("{}_{}.{}", stem, n, ext);
            let extra_path = output_dir.join(&extra_filename);
            fs::write(&extra_path, format!("v{}", n).as_bytes()).unwrap();
        }

        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::Original,
            &ConflictResolution::Rename,
            1,
        );

        match result {
            OutputPathResult::Write(path) => {
                let result_filename = path.file_name().unwrap().to_str().unwrap();

                // The result should be {stem}_{N}.{ext} where N >= 2
                let expected_n = extra_files + 2; // next available number
                let expected_filename = format!("{}_{}.{}", stem, expected_n, ext);

                prop_assert_eq!(
                    result_filename,
                    expected_filename.as_str(),
                    "Rename conflict resolution should produce {{stem}}_{{N}}.{{ext}} with N≥2.\n\
                     Expected N={}, got filename: {}",
                    expected_n,
                    result_filename
                );

                // Verify the result path does not exist (it's a new unique name)
                prop_assert!(
                    !path.exists(),
                    "Renamed output path should not already exist"
                );
            }
            OutputPathResult::Skip => {
                prop_assert!(false, "Rename conflict resolution should never produce Skip");
            }
        }
    }

    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_conflict_skip_allows_new_file(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"source data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        // Do NOT create an existing file - output location is empty
        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::Original,
            &ConflictResolution::Skip,
            1,
        );

        // When no existing file, Skip should still produce a Write result
        let expected_filename = format!("{}.{}", stem, format_extension(&output_format));
        let expected_path = output_dir.join(&expected_filename);

        prop_assert_eq!(
            result,
            OutputPathResult::Write(expected_path),
            "Skip conflict resolution should allow writing when no existing file"
        );
    }

    /// **Validates: Requirements 5.4**
    #[test]
    fn prop_conflict_rename_no_conflict_uses_original(
        stem in arb_filename_stem(),
        src_ext in arb_extension(),
        output_format in arb_file_format(),
    ) {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join(format!("{}.{}", stem, src_ext));
        fs::write(&source, b"source data").unwrap();

        let output_dir = tmp.path().join("out");
        fs::create_dir_all(&output_dir).unwrap();

        // Do NOT create an existing file
        let result = generate_output_path(
            &source,
            Some(output_dir.as_path()),
            &output_format,
            &FileNamingPattern::Original,
            &ConflictResolution::Rename,
            1,
        );

        // When no conflict exists, Rename should use the original name
        let expected_filename = format!("{}.{}", stem, format_extension(&output_format));
        let expected_path = output_dir.join(&expected_filename);

        prop_assert_eq!(
            result,
            OutputPathResult::Write(expected_path),
            "Rename conflict resolution should use original name when no conflict exists"
        );
    }
}
