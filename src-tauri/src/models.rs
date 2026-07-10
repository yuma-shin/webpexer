use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ファイルフォーマット
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    // Image formats
    Png,
    Jpeg,
    Webp,
    Gif,
    Bmp,
    Tiff,
    Avif,
    Ico,
    // Text formats
    Json,
    Yaml,
    Toml,
    Xml,
    Csv,
    Markdown,
    #[serde(rename = "plaintext")]
    PlainText,
}

/// フォーマットカテゴリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormatCategory {
    Image,
    Text,
}

/// フォーマット情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatInfo {
    pub format: FileFormat,
    pub name: String,
    pub extensions: Vec<String>,
    pub category: FormatCategory,
}

/// 変換パラメータ（フロントエンドから受け取る）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionParams {
    pub input_paths: Vec<PathBuf>,
    pub output_format: FileFormat,
    pub output_dir: Option<PathBuf>,
    pub options: ConversionOptions,
}

/// 変換オプション
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionOptions {
    pub quality: u8,
    pub resize: Option<ResizeOptions>,
    pub encoding: Option<TextEncoding>,
    pub csv_delimiter: Option<CsvDelimiter>,
    pub file_naming: FileNamingPattern,
    pub conflict_resolution: ConflictResolution,
    pub delete_source: bool,
}

/// リサイズオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResizeOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub maintain_aspect_ratio: bool,
}

/// テキストエンコーディング
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextEncoding {
    Utf8,
    ShiftJis,
    EucJp,
}

/// CSV区切り文字
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CsvDelimiter {
    Comma,
    Tab,
    Semicolon,
}

/// ファイル命名パターン
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileNamingPattern {
    Original,
    OriginalSequential,
    OriginalDatetime,
}

/// 衝突解決方式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictResolution {
    Overwrite,
    Skip,
    Rename,
}

/// 変換進捗イベント
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressEvent {
    pub current_file: String,
    pub processed_count: u32,
    pub total_count: u32,
    pub status: ProgressStatus,
    pub error: Option<String>,
}

/// 進捗ステータス
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProgressStatus {
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// 変換結果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionResult {
    pub success_count: u32,
    pub failed_count: u32,
    pub skipped_count: u32,
    pub failed_files: Vec<FailedFileInfo>,
    pub output_dir: PathBuf,
}

/// 失敗ファイル情報
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedFileInfo {
    pub file_name: String,
    pub error: String,
}

/// ディスク容量チェック結果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskSpaceCheck {
    pub has_enough_space: bool,
    pub required_bytes: u64,
    pub available_bytes: u64,
}

/// パス検証結果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathValidation {
    pub is_valid: bool,
    pub is_writable: bool,
    pub error: Option<String>,
}

/// 大容量ファイル情報（100MB超ファイルの確認ダイアログ用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeFileInfo {
    pub name: String,
    pub size_mb: f64,
}
