use std::path::{Path, PathBuf};

use chrono::Local;

use crate::models::{ConflictResolution, FileFormat, FileNamingPattern};

/// 出力パス生成の結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputPathResult {
    /// ファイルを書き込む（パスを返す）
    Write(PathBuf),
    /// スキップ（ファイルが既に存在し、Skip モードが選択されている場合）
    Skip,
}

/// FileFormat から拡張子文字列を取得する
pub fn format_extension(format: &FileFormat) -> &'static str {
    match format {
        FileFormat::Png => "png",
        FileFormat::Jpeg => "jpg",
        FileFormat::Webp => "webp",
        FileFormat::Gif => "gif",
        FileFormat::Bmp => "bmp",
        FileFormat::Tiff => "tiff",
        FileFormat::Avif => "avif",
        FileFormat::Ico => "ico",
        FileFormat::Json => "json",
        FileFormat::Yaml => "yaml",
        FileFormat::Toml => "toml",
        FileFormat::Xml => "xml",
        FileFormat::Csv => "csv",
        FileFormat::Markdown => "md",
        FileFormat::PlainText => "txt",
    }
}

/// FileFormat からフォーマット名文字列を取得する（サブフォルダ名に使用）
pub fn format_name(format: &FileFormat) -> &'static str {
    match format {
        FileFormat::Png => "png",
        FileFormat::Jpeg => "jpeg",
        FileFormat::Webp => "webp",
        FileFormat::Gif => "gif",
        FileFormat::Bmp => "bmp",
        FileFormat::Tiff => "tiff",
        FileFormat::Avif => "avif",
        FileFormat::Ico => "ico",
        FileFormat::Json => "json",
        FileFormat::Yaml => "yaml",
        FileFormat::Toml => "toml",
        FileFormat::Xml => "xml",
        FileFormat::Csv => "csv",
        FileFormat::Markdown => "markdown",
        FileFormat::PlainText => "plaintext",
    }
}

/// ファイル命名パターンに基づいてファイル名ステム（拡張子なし）を生成する
fn generate_filename_stem(
    original_stem: &str,
    naming: &FileNamingPattern,
    sequence_num: u32,
) -> String {
    match naming {
        FileNamingPattern::Original => original_stem.to_string(),
        FileNamingPattern::OriginalSequential => {
            format!("{}_{:03}", original_stem, sequence_num)
        }
        FileNamingPattern::OriginalDatetime => {
            let now = Local::now();
            let datetime_str = now.format("%Y%m%d_%H%M%S").to_string();
            format!("{}_{}", original_stem, datetime_str)
        }
    }
}

/// 衝突解決ロジック: 既にファイルが存在する場合の出力パスを決定する
fn resolve_conflict(base_path: &Path, conflict: &ConflictResolution) -> OutputPathResult {
    match conflict {
        ConflictResolution::Overwrite => OutputPathResult::Write(base_path.to_path_buf()),
        ConflictResolution::Skip => {
            if base_path.exists() {
                OutputPathResult::Skip
            } else {
                OutputPathResult::Write(base_path.to_path_buf())
            }
        }
        ConflictResolution::Rename => {
            if !base_path.exists() {
                return OutputPathResult::Write(base_path.to_path_buf());
            }

            let parent = base_path.parent().unwrap_or(Path::new(""));
            let stem = base_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let ext = base_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let mut n: u32 = 2;
            loop {
                let new_name = if ext.is_empty() {
                    format!("{}_{}", stem, n)
                } else {
                    format!("{}_{}.{}", stem, n, ext)
                };
                let candidate = parent.join(&new_name);
                if !candidate.exists() {
                    return OutputPathResult::Write(candidate);
                }
                n += 1;
            }
        }
    }
}

/// 出力先ディレクトリを決定する
///
/// - `output_dir` が指定されていれば、そのディレクトリを使用
/// - 指定されていなければ、ソースファイルの親ディレクトリ内に `{format_name}/` サブフォルダを作成
fn determine_output_dir(
    source: &Path,
    output_dir: Option<&Path>,
    format: &FileFormat,
) -> PathBuf {
    match output_dir {
        Some(dir) => dir.to_path_buf(),
        None => {
            let source_dir = source.parent().unwrap_or(Path::new("."));
            source_dir.join(format_name(format))
        }
    }
}

/// 出力パスを生成する
///
/// # Arguments
///
/// * `source` - ソースファイルのパス
/// * `output_dir` - 出力ディレクトリ（Noneの場合はデフォルトパスを使用）
/// * `format` - 出力フォーマット
/// * `naming` - ファイル命名パターン
/// * `conflict` - 衝突解決方式
/// * `sequence_num` - 連番（OriginalSequential パターン時に使用）
///
/// # Returns
///
/// `OutputPathResult::Write(path)` - 書き込み先パス
/// `OutputPathResult::Skip` - スキップ（ファイルが既に存在し、Skip モードの場合）
pub fn generate_output_path(
    source: &Path,
    output_dir: Option<&Path>,
    format: &FileFormat,
    naming: &FileNamingPattern,
    conflict: &ConflictResolution,
    sequence_num: u32,
) -> OutputPathResult {
    // 出力ディレクトリの決定
    let dir = determine_output_dir(source, output_dir, format);

    // 元ファイルのステム取得
    let original_stem = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    // ファイル名ステム生成（命名パターンに基づく）
    let stem = generate_filename_stem(original_stem, naming, sequence_num);

    // 拡張子を取得
    let ext = format_extension(format);

    // 完全な出力パスを構築
    let filename = format!("{}.{}", stem, ext);
    let output_path = dir.join(&filename);

    // 衝突解決
    resolve_conflict(&output_path, conflict)
}

/// 出力ディレクトリが存在しない場合に作成する
///
/// # Returns
///
/// 成功時は `Ok(())`, 失敗時はエラーメッセージを返す
pub fn ensure_output_dir(
    source: &Path,
    output_dir: Option<&Path>,
    format: &FileFormat,
) -> Result<PathBuf, String> {
    let dir = determine_output_dir(source, output_dir, format);

    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| {
            format!(
                "出力ディレクトリの作成に失敗しました: {} - {}",
                dir.display(),
                e
            )
        })?;
    }

    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_format_extension() {
        assert_eq!(format_extension(&FileFormat::Png), "png");
        assert_eq!(format_extension(&FileFormat::Jpeg), "jpg");
        assert_eq!(format_extension(&FileFormat::Webp), "webp");
        assert_eq!(format_extension(&FileFormat::Gif), "gif");
        assert_eq!(format_extension(&FileFormat::Bmp), "bmp");
        assert_eq!(format_extension(&FileFormat::Tiff), "tiff");
        assert_eq!(format_extension(&FileFormat::Avif), "avif");
        assert_eq!(format_extension(&FileFormat::Ico), "ico");
        assert_eq!(format_extension(&FileFormat::Json), "json");
        assert_eq!(format_extension(&FileFormat::Yaml), "yaml");
        assert_eq!(format_extension(&FileFormat::Toml), "toml");
        assert_eq!(format_extension(&FileFormat::Xml), "xml");
        assert_eq!(format_extension(&FileFormat::Csv), "csv");
        assert_eq!(format_extension(&FileFormat::Markdown), "md");
        assert_eq!(format_extension(&FileFormat::PlainText), "txt");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(format_name(&FileFormat::Png), "png");
        assert_eq!(format_name(&FileFormat::Jpeg), "jpeg");
        assert_eq!(format_name(&FileFormat::Yaml), "yaml");
        assert_eq!(format_name(&FileFormat::Markdown), "markdown");
        assert_eq!(format_name(&FileFormat::PlainText), "plaintext");
    }

    #[test]
    fn test_generate_output_path_original_naming_no_output_dir() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("photo.png");
        fs::write(&source, b"dummy").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Jpeg,
            &FileNamingPattern::Original,
            &ConflictResolution::Overwrite,
            1,
        );

        let expected = tmp.path().join("jpeg").join("photo.jpg");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_generate_output_path_with_output_dir() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("image.webp");
        fs::write(&source, b"dummy").unwrap();

        let output_dir = tmp.path().join("output");
        fs::create_dir_all(&output_dir).unwrap();

        let result = generate_output_path(
            &source,
            Some(&output_dir),
            &FileFormat::Png,
            &FileNamingPattern::Original,
            &ConflictResolution::Overwrite,
            1,
        );

        let expected = output_dir.join("image.png");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_generate_output_path_sequential_naming() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("doc.json");
        fs::write(&source, b"{}").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Yaml,
            &FileNamingPattern::OriginalSequential,
            &ConflictResolution::Overwrite,
            1,
        );

        let expected = tmp.path().join("yaml").join("doc_001.yaml");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_generate_output_path_sequential_naming_multi_digit() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("file.txt");
        fs::write(&source, b"text").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Markdown,
            &FileNamingPattern::OriginalSequential,
            &ConflictResolution::Overwrite,
            42,
        );

        let expected = tmp.path().join("markdown").join("file_042.md");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_generate_output_path_datetime_naming() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("data.csv");
        fs::write(&source, b"a,b\n1,2").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Json,
            &FileNamingPattern::OriginalDatetime,
            &ConflictResolution::Overwrite,
            1,
        );

        // Verify it's a Write result with expected pattern
        match result {
            OutputPathResult::Write(path) => {
                let filename = path.file_name().unwrap().to_str().unwrap();
                // Should start with "data_" and end with ".json"
                assert!(filename.starts_with("data_"));
                assert!(filename.ends_with(".json"));
                // Check datetime pattern: data_YYYYMMDD_HHMMSS.json
                let stem = path.file_stem().unwrap().to_str().unwrap();
                let parts: Vec<&str> = stem.splitn(2, '_').collect();
                assert_eq!(parts[0], "data");
                // The datetime part should be like "20240101_120000"
                assert_eq!(parts[1].len(), 15); // yyyyMMdd_HHmmss = 15 chars
            }
            OutputPathResult::Skip => panic!("Expected Write, got Skip"),
        }
    }

    #[test]
    fn test_conflict_resolution_overwrite() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("file.png");
        fs::write(&source, b"source").unwrap();

        // Create existing output file
        let output_dir = tmp.path().join("jpeg");
        fs::create_dir_all(&output_dir).unwrap();
        let existing = output_dir.join("file.jpg");
        fs::write(&existing, b"existing").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Jpeg,
            &FileNamingPattern::Original,
            &ConflictResolution::Overwrite,
            1,
        );

        // Overwrite: still returns the same path
        assert_eq!(result, OutputPathResult::Write(existing));
    }

    #[test]
    fn test_conflict_resolution_skip() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("file.png");
        fs::write(&source, b"source").unwrap();

        // Create existing output file
        let output_dir = tmp.path().join("jpeg");
        fs::create_dir_all(&output_dir).unwrap();
        let existing = output_dir.join("file.jpg");
        fs::write(&existing, b"existing").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Jpeg,
            &FileNamingPattern::Original,
            &ConflictResolution::Skip,
            1,
        );

        assert_eq!(result, OutputPathResult::Skip);
    }

    #[test]
    fn test_conflict_resolution_skip_no_existing_file() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("file.png");
        fs::write(&source, b"source").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Jpeg,
            &FileNamingPattern::Original,
            &ConflictResolution::Skip,
            1,
        );

        let expected = tmp.path().join("jpeg").join("file.jpg");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_conflict_resolution_rename() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("file.png");
        fs::write(&source, b"source").unwrap();

        // Create existing output file
        let output_dir = tmp.path().join("jpeg");
        fs::create_dir_all(&output_dir).unwrap();
        let existing = output_dir.join("file.jpg");
        fs::write(&existing, b"existing").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Jpeg,
            &FileNamingPattern::Original,
            &ConflictResolution::Rename,
            1,
        );

        // Should get file_2.jpg since file.jpg already exists
        let expected = output_dir.join("file_2.jpg");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_conflict_resolution_rename_multiple() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("file.png");
        fs::write(&source, b"source").unwrap();

        // Create existing output files: file.jpg, file_2.jpg, file_3.jpg
        let output_dir = tmp.path().join("jpeg");
        fs::create_dir_all(&output_dir).unwrap();
        fs::write(output_dir.join("file.jpg"), b"v1").unwrap();
        fs::write(output_dir.join("file_2.jpg"), b"v2").unwrap();
        fs::write(output_dir.join("file_3.jpg"), b"v3").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Jpeg,
            &FileNamingPattern::Original,
            &ConflictResolution::Rename,
            1,
        );

        // Should get file_4.jpg since file.jpg, file_2.jpg, file_3.jpg exist
        let expected = output_dir.join("file_4.jpg");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_conflict_resolution_rename_no_existing() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("file.png");
        fs::write(&source, b"source").unwrap();

        let result = generate_output_path(
            &source,
            None,
            &FileFormat::Jpeg,
            &FileNamingPattern::Original,
            &ConflictResolution::Rename,
            1,
        );

        // No conflict, so original name is used
        let expected = tmp.path().join("jpeg").join("file.jpg");
        assert_eq!(result, OutputPathResult::Write(expected));
    }

    #[test]
    fn test_ensure_output_dir_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("image.png");
        fs::write(&source, b"data").unwrap();

        let result = ensure_output_dir(&source, None, &FileFormat::Jpeg);
        assert!(result.is_ok());

        let dir = result.unwrap();
        assert!(dir.exists());
        assert_eq!(dir, tmp.path().join("jpeg"));
    }

    #[test]
    fn test_ensure_output_dir_with_custom_output() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("image.png");
        fs::write(&source, b"data").unwrap();

        let custom_dir = tmp.path().join("custom_output");

        let result = ensure_output_dir(&source, Some(&custom_dir), &FileFormat::Png);
        assert!(result.is_ok());

        let dir = result.unwrap();
        assert!(dir.exists());
        assert_eq!(dir, custom_dir);
    }

    #[test]
    fn test_ensure_output_dir_already_exists() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("image.png");
        fs::write(&source, b"data").unwrap();

        let existing_dir = tmp.path().join("jpeg");
        fs::create_dir_all(&existing_dir).unwrap();

        let result = ensure_output_dir(&source, None, &FileFormat::Jpeg);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), existing_dir);
    }
}
