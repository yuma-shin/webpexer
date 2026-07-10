use std::fs;
use std::path::{Path, PathBuf};

use crate::errors::ConversionError;
use crate::models::FileFormat;

/// バリデーション結果: 有効ファイル（フォーマット付き）と無効ファイル
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// 有効なファイルとその検出されたフォーマット
    pub valid_files: Vec<(PathBuf, FileFormat)>,
    /// 無効またはスキップされたファイル（パスとスキップ理由）
    pub invalid_files: Vec<(PathBuf, String)>,
}

/// 入力バリデーション
pub struct Validator;

impl Validator {
    /// 入力ファイルの存在確認とフォーマット判定
    ///
    /// 各パスについて:
    /// - ディレクトリの場合は再帰的にファイルを収集
    /// - 存在するか確認
    /// - 拡張子からフォーマットを判定
    /// - 結果を ValidationResult にまとめて返す
    pub fn validate_input_files(paths: &[PathBuf]) -> ValidationResult {
        let mut valid_files = Vec::new();
        let mut invalid_files = Vec::new();

        // First, expand any directories into individual files
        let mut expanded_paths: Vec<PathBuf> = Vec::new();
        for path in paths {
            if path.is_dir() {
                Self::collect_files_from_dir(path, &mut expanded_paths);
            } else {
                expanded_paths.push(path.clone());
            }
        }

        // Then validate each expanded file
        for path in &expanded_paths {
            if !path.exists() {
                invalid_files.push((
                    path.clone(),
                    format!("ファイルが存在しません: {}", path.display()),
                ));
                continue;
            }

            if !path.is_file() {
                // Skip non-file entries (shouldn't happen after expansion, but safety check)
                continue;
            }

            match Self::detect_format(path) {
                Some(format) => {
                    valid_files.push((path.clone(), format));
                }
                None => {
                    let ext = path
                        .extension()
                        .map(|e| e.to_string_lossy().to_string())
                        .unwrap_or_else(|| "なし".to_string());
                    invalid_files.push((
                        path.clone(),
                        format!("非対応フォーマット (拡張子: {})", ext),
                    ));
                }
            }
        }

        ValidationResult {
            valid_files,
            invalid_files,
        }
    }

    /// ディレクトリ内のファイルを再帰的に収集する
    fn collect_files_from_dir(dir: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_files_from_dir(&path, files);
                } else if path.is_file() {
                    files.push(path);
                }
            }
        }
    }

    /// 出力先パスの書き込み権限確認
    ///
    /// - パスが存在するか、または作成可能か確認
    /// - 書き込み権限をテンポラリファイル作成で検証
    pub fn validate_output_path(path: &Path) -> Result<(), ConversionError> {
        // パスが存在しない場合、作成を試みる
        if !path.exists() {
            fs::create_dir_all(path).map_err(|e| ConversionError::DirectoryCreateError {
                path: path.display().to_string(),
                detail: e.to_string(),
            })?;
        }

        // ディレクトリであることを確認
        if !path.is_dir() {
            return Err(ConversionError::PermissionError {
                path: path.display().to_string(),
            });
        }

        // 書き込み権限をテンポラリファイル作成で確認
        let test_file = path.join(".webpexer_write_test");
        match fs::write(&test_file, b"test") {
            Ok(_) => {
                // テストファイルを削除（失敗しても問題なし）
                let _ = fs::remove_file(&test_file);
                Ok(())
            }
            Err(_) => Err(ConversionError::PermissionError {
                path: path.display().to_string(),
            }),
        }
    }

    /// 入力ファイル合計サイズと出力先空き容量の比較
    ///
    /// input_size: 入力ファイルの合計バイト数
    /// output_path: 出力先ディレクトリパス
    pub fn check_disk_space(input_size: u64, output_path: &Path) -> Result<(), ConversionError> {
        let available = Self::get_available_space(output_path);

        if input_size > available {
            return Err(ConversionError::DiskSpaceError {
                required_bytes: input_size,
                available_bytes: available,
            });
        }

        Ok(())
    }

    /// テキストファイルのサイズ上限チェック
    ///
    /// max_size: 上限バイト数（50MB = 50 * 1024 * 1024）
    pub fn validate_file_size(path: &Path, max_size: u64) -> Result<(), ConversionError> {
        let metadata = fs::metadata(path).map_err(|e| ConversionError::FileReadError {
            file_name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            detail: e.to_string(),
        })?;

        let file_size = metadata.len();
        if file_size > max_size {
            let size_mb = file_size / (1024 * 1024);
            let limit_mb = max_size / (1024 * 1024);
            return Err(ConversionError::FileSizeLimitError {
                file_name: path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                size_mb,
                limit_mb,
            });
        }

        Ok(())
    }

    /// 拡張子からファイルフォーマットを検出
    fn detect_format(path: &Path) -> Option<FileFormat> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            // Image formats
            "png" => Some(FileFormat::Png),
            "jpg" | "jpeg" => Some(FileFormat::Jpeg),
            "webp" => Some(FileFormat::Webp),
            "gif" => Some(FileFormat::Gif),
            "bmp" => Some(FileFormat::Bmp),
            "tiff" | "tif" => Some(FileFormat::Tiff),
            "avif" => Some(FileFormat::Avif),
            "ico" => Some(FileFormat::Ico),
            // Text formats
            "json" => Some(FileFormat::Json),
            "yaml" | "yml" => Some(FileFormat::Yaml),
            "toml" => Some(FileFormat::Toml),
            "xml" => Some(FileFormat::Xml),
            "csv" => Some(FileFormat::Csv),
            "md" | "markdown" => Some(FileFormat::Markdown),
            "txt" => Some(FileFormat::PlainText),
            _ => None,
        }
    }

    /// 出力先パスの空きディスク容量を取得
    ///
    /// プラットフォーム固有の実装を使用
    #[cfg(target_os = "windows")]
    pub fn get_available_space(path: &Path) -> u64 {
        use std::os::windows::ffi::OsStrExt;

        // 既存のパスを探す（存在しないパスの場合は親をたどる）
        let check_path = Self::find_existing_ancestor(path);

        let wide_path: Vec<u16> = check_path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes_available: u64 = 0;
        let mut total_number_of_bytes: u64 = 0;
        let mut total_number_of_free_bytes: u64 = 0;

        unsafe {
            let result = windows_sys::Win32::Storage::FileSystem::GetDiskFreeSpaceExW(
                wide_path.as_ptr(),
                &mut free_bytes_available,
                &mut total_number_of_bytes,
                &mut total_number_of_free_bytes,
            );
            if result != 0 {
                free_bytes_available
            } else {
                0
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_available_space(path: &Path) -> u64 {
        use std::ffi::CString;

        // 既存のパスを探す（存在しないパスの場合は親をたどる）
        let check_path = Self::find_existing_ancestor(path);

        let c_path = match CString::new(check_path.to_string_lossy().as_bytes()) {
            Ok(p) => p,
            Err(_) => return 0,
        };

        unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            if libc::statvfs(c_path.as_ptr(), &mut stat) == 0 {
                stat.f_bavail as u64 * stat.f_frsize as u64
            } else {
                0
            }
        }
    }

    /// 既存の祖先ディレクトリを探す
    fn find_existing_ancestor(path: &Path) -> PathBuf {
        let mut current = path.to_path_buf();
        while !current.exists() {
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_format_image_formats() {
        let cases = vec![
            ("test.png", Some(FileFormat::Png)),
            ("test.jpg", Some(FileFormat::Jpeg)),
            ("test.jpeg", Some(FileFormat::Jpeg)),
            ("test.webp", Some(FileFormat::Webp)),
            ("test.gif", Some(FileFormat::Gif)),
            ("test.bmp", Some(FileFormat::Bmp)),
            ("test.tiff", Some(FileFormat::Tiff)),
            ("test.tif", Some(FileFormat::Tiff)),
            ("test.avif", Some(FileFormat::Avif)),
            ("test.ico", Some(FileFormat::Ico)),
        ];

        for (filename, expected) in cases {
            let path = Path::new(filename);
            assert_eq!(
                Validator::detect_format(path),
                expected,
                "Failed for: {}",
                filename
            );
        }
    }

    #[test]
    fn test_detect_format_text_formats() {
        let cases = vec![
            ("test.json", Some(FileFormat::Json)),
            ("test.yaml", Some(FileFormat::Yaml)),
            ("test.yml", Some(FileFormat::Yaml)),
            ("test.toml", Some(FileFormat::Toml)),
            ("test.xml", Some(FileFormat::Xml)),
            ("test.csv", Some(FileFormat::Csv)),
            ("test.md", Some(FileFormat::Markdown)),
            ("test.markdown", Some(FileFormat::Markdown)),
            ("test.txt", Some(FileFormat::PlainText)),
        ];

        for (filename, expected) in cases {
            let path = Path::new(filename);
            assert_eq!(
                Validator::detect_format(path),
                expected,
                "Failed for: {}",
                filename
            );
        }
    }

    #[test]
    fn test_detect_format_unsupported() {
        let cases = vec!["test.exe", "test.doc", "test.pdf", "test"];
        for filename in cases {
            let path = Path::new(filename);
            assert_eq!(
                Validator::detect_format(path),
                None,
                "Should be None for: {}",
                filename
            );
        }
    }

    #[test]
    fn test_detect_format_case_insensitive() {
        let cases = vec![
            ("test.PNG", Some(FileFormat::Png)),
            ("test.JPG", Some(FileFormat::Jpeg)),
            ("test.Json", Some(FileFormat::Json)),
        ];

        for (filename, expected) in cases {
            let path = Path::new(filename);
            assert_eq!(
                Validator::detect_format(path),
                expected,
                "Failed for: {}",
                filename
            );
        }
    }

    #[test]
    fn test_validate_input_files_valid() {
        let dir = TempDir::new().unwrap();
        let png_path = dir.path().join("test.png");
        let json_path = dir.path().join("data.json");
        fs::write(&png_path, b"fake png").unwrap();
        fs::write(&json_path, b"{}").unwrap();

        let result = Validator::validate_input_files(&[png_path.clone(), json_path.clone()]);

        assert_eq!(result.valid_files.len(), 2);
        assert_eq!(result.invalid_files.len(), 0);
        assert_eq!(result.valid_files[0].1, FileFormat::Png);
        assert_eq!(result.valid_files[1].1, FileFormat::Json);
    }

    #[test]
    fn test_validate_input_files_nonexistent() {
        let path = PathBuf::from("/nonexistent/file.png");
        let result = Validator::validate_input_files(&[path.clone()]);

        assert_eq!(result.valid_files.len(), 0);
        assert_eq!(result.invalid_files.len(), 1);
        assert!(result.invalid_files[0].1.contains("存在しません"));
    }

    #[test]
    fn test_validate_input_files_unsupported_format() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("file.xyz");
        fs::write(&path, b"data").unwrap();

        let result = Validator::validate_input_files(&[path.clone()]);

        assert_eq!(result.valid_files.len(), 0);
        assert_eq!(result.invalid_files.len(), 1);
        assert!(result.invalid_files[0].1.contains("非対応フォーマット"));
    }

    #[test]
    fn test_validate_input_files_mixed() {
        let dir = TempDir::new().unwrap();
        let valid_path = dir.path().join("image.webp");
        let invalid_path = dir.path().join("doc.pdf");
        let missing_path = PathBuf::from("/missing/file.png");
        fs::write(&valid_path, b"fake webp").unwrap();
        fs::write(&invalid_path, b"fake pdf").unwrap();

        let result =
            Validator::validate_input_files(&[valid_path, invalid_path, missing_path]);

        assert_eq!(result.valid_files.len(), 1);
        assert_eq!(result.invalid_files.len(), 2);
    }

    #[test]
    fn test_validate_output_path_valid() {
        let dir = TempDir::new().unwrap();
        let result = Validator::validate_output_path(dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_output_path_creates_dir() {
        let dir = TempDir::new().unwrap();
        let new_dir = dir.path().join("new_subdir");
        assert!(!new_dir.exists());

        let result = Validator::validate_output_path(&new_dir);
        assert!(result.is_ok());
        assert!(new_dir.exists());
    }

    #[test]
    fn test_validate_file_size_within_limit() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("small.txt");
        fs::write(&path, "hello").unwrap();

        let max_size = 50 * 1024 * 1024; // 50MB
        let result = Validator::validate_file_size(&path, max_size);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_size_exceeds_limit() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("large.txt");
        // Create a file slightly over 1MB for testing (avoid creating 50MB files in tests)
        let data = vec![0u8; 2 * 1024 * 1024]; // 2MB
        fs::write(&path, &data).unwrap();

        let max_size = 1 * 1024 * 1024; // 1MB limit for test
        let result = Validator::validate_file_size(&path, max_size);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConversionError::FileSizeLimitError {
                file_name,
                size_mb,
                limit_mb,
            } => {
                assert_eq!(file_name, "large.txt");
                assert_eq!(size_mb, 2); // 2MB
                assert_eq!(limit_mb, 1); // 1MB limit
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_validate_file_size_nonexistent() {
        let path = Path::new("/nonexistent/file.txt");
        let result = Validator::validate_file_size(path, 50 * 1024 * 1024);
        assert!(result.is_err());

        match result.unwrap_err() {
            ConversionError::FileReadError { .. } => {}
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_check_disk_space_sufficient() {
        let dir = TempDir::new().unwrap();
        // 1 byte should always have enough space
        let result = Validator::check_disk_space(1, dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_disk_space_insufficient() {
        let dir = TempDir::new().unwrap();
        // u64::MAX should never have enough space
        let result = Validator::check_disk_space(u64::MAX, dir.path());
        assert!(result.is_err());

        match result.unwrap_err() {
            ConversionError::DiskSpaceError {
                required_bytes,
                available_bytes: _,
            } => {
                assert_eq!(required_bytes, u64::MAX);
            }
            e => panic!("Unexpected error type: {:?}", e),
        }
    }

    #[test]
    fn test_validate_input_files_directory_expansion() {
        let dir = TempDir::new().unwrap();
        // Create a subdirectory with files
        let subdir = dir.path().join("images");
        fs::create_dir(&subdir).unwrap();
        let png_path = subdir.join("photo.png");
        let jpg_path = subdir.join("pic.jpg");
        let unsupported_path = subdir.join("readme.pdf");
        fs::write(&png_path, b"fake png").unwrap();
        fs::write(&jpg_path, b"fake jpg").unwrap();
        fs::write(&unsupported_path, b"fake pdf").unwrap();

        // Pass the directory path - it should expand into individual files
        let result = Validator::validate_input_files(&[subdir]);

        assert_eq!(result.valid_files.len(), 2);
        assert_eq!(result.invalid_files.len(), 1);
        // Check that supported formats were detected
        let formats: Vec<_> = result.valid_files.iter().map(|(_, f)| f.clone()).collect();
        assert!(formats.contains(&FileFormat::Png));
        assert!(formats.contains(&FileFormat::Jpeg));
    }

    #[test]
    fn test_validate_input_files_nested_directory_expansion() {
        let dir = TempDir::new().unwrap();
        // Create nested subdirectories
        let subdir = dir.path().join("project");
        let nested = subdir.join("assets");
        fs::create_dir_all(&nested).unwrap();
        let png_path = subdir.join("icon.png");
        let webp_path = nested.join("banner.webp");
        fs::write(&png_path, b"fake png").unwrap();
        fs::write(&webp_path, b"fake webp").unwrap();

        // Pass the top-level directory
        let result = Validator::validate_input_files(&[subdir]);

        assert_eq!(result.valid_files.len(), 2);
        assert_eq!(result.invalid_files.len(), 0);
    }

    #[test]
    fn test_validate_input_files_mixed_files_and_directories() {
        let dir = TempDir::new().unwrap();
        // Create a file at top level
        let top_file = dir.path().join("top.json");
        fs::write(&top_file, b"{}").unwrap();
        // Create a subdirectory with a file
        let subdir = dir.path().join("sub");
        fs::create_dir(&subdir).unwrap();
        let sub_file = subdir.join("data.yaml");
        fs::write(&sub_file, b"key: value").unwrap();

        // Pass both a file and a directory
        let result = Validator::validate_input_files(&[top_file, subdir]);

        assert_eq!(result.valid_files.len(), 2);
        assert_eq!(result.invalid_files.len(), 0);
    }
}
