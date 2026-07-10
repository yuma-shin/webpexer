use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// 変換エラー
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
pub enum ConversionError {
    #[error("ファイル読み取りエラー: {file_name} - {detail}")]
    FileReadError { file_name: String, detail: String },

    #[error("デコードエラー: {file_name} - {detail}")]
    DecodeError { file_name: String, detail: String },

    #[error("エンコードエラー: {file_name} - {detail}")]
    EncodeError { file_name: String, detail: String },

    #[error("パースエラー: {file_name} 行{line:?} - {detail}")]
    ParseError {
        file_name: String,
        line: Option<u32>,
        detail: String,
    },

    #[error("バリデーションエラー: {detail}")]
    ValidationError { detail: String },

    #[error("ディスク容量不足: 必要={required_bytes}, 空き={available_bytes}")]
    DiskSpaceError {
        required_bytes: u64,
        available_bytes: u64,
    },

    #[error("権限エラー: {path}")]
    PermissionError { path: String },

    #[error("ファイルサイズ超過: {file_name} ({size_mb}MB > {limit_mb}MB)")]
    FileSizeLimitError {
        file_name: String,
        size_mb: u64,
        limit_mb: u64,
    },

    #[error("非対応変換ペア: {from} → {to}")]
    UnsupportedConversion { from: String, to: String },

    #[error("キャンセルされました")]
    Cancelled,

    #[error("ファイル書き込みエラー: {file_name} - {detail}")]
    FileWriteError { file_name: String, detail: String },

    #[error("フォルダ作成失敗: {path} - {detail}")]
    DirectoryCreateError { path: String, detail: String },
}

impl ConversionError {
    /// フロントエンドの i18n キーを返す。
    /// フロントエンド側では `t(error_key, error_params)` で翻訳メッセージを生成する。
    pub fn error_key(&self) -> &'static str {
        match self {
            ConversionError::FileReadError { .. } => "errors.fileRead",
            ConversionError::DecodeError { .. } => "errors.decode",
            ConversionError::EncodeError { .. } => "errors.encode",
            ConversionError::ParseError { .. } => "errors.parseError",
            ConversionError::ValidationError { .. } => "errors.validation",
            ConversionError::DiskSpaceError { .. } => "errors.diskSpace",
            ConversionError::PermissionError { .. } => "errors.permission",
            ConversionError::FileSizeLimitError { .. } => "errors.fileSize",
            ConversionError::UnsupportedConversion { .. } => "errors.unsupported",
            ConversionError::Cancelled => "errors.cancelled",
            ConversionError::FileWriteError { .. } => "errors.fileWrite",
            ConversionError::DirectoryCreateError { .. } => "errors.directoryCreate",
        }
    }

    /// i18n 補間パラメータを返す。
    /// フロントエンド側で `t(key, params)` として使用する。
    pub fn error_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        match self {
            ConversionError::FileReadError { file_name, detail } => {
                params.insert("fileName".to_string(), file_name.clone());
                params.insert("detail".to_string(), detail.clone());
            }
            ConversionError::DecodeError { file_name, detail } => {
                params.insert("fileName".to_string(), file_name.clone());
                params.insert("detail".to_string(), detail.clone());
            }
            ConversionError::EncodeError { file_name, detail } => {
                params.insert("fileName".to_string(), file_name.clone());
                params.insert("detail".to_string(), detail.clone());
            }
            ConversionError::ParseError {
                file_name,
                line,
                detail,
            } => {
                params.insert("fileName".to_string(), file_name.clone());
                params.insert(
                    "line".to_string(),
                    line.map(|l| l.to_string()).unwrap_or_default(),
                );
                params.insert("detail".to_string(), detail.clone());
            }
            ConversionError::ValidationError { detail } => {
                params.insert("detail".to_string(), detail.clone());
            }
            ConversionError::DiskSpaceError {
                required_bytes,
                available_bytes,
            } => {
                params.insert(
                    "required".to_string(),
                    format_bytes(*required_bytes),
                );
                params.insert(
                    "available".to_string(),
                    format_bytes(*available_bytes),
                );
            }
            ConversionError::PermissionError { path } => {
                params.insert("path".to_string(), path.clone());
            }
            ConversionError::FileSizeLimitError {
                file_name,
                size_mb,
                limit_mb,
            } => {
                params.insert("fileName".to_string(), file_name.clone());
                params.insert("size".to_string(), size_mb.to_string());
                params.insert("limit".to_string(), limit_mb.to_string());
            }
            ConversionError::UnsupportedConversion { from, to } => {
                params.insert("from".to_string(), from.clone());
                params.insert("to".to_string(), to.clone());
            }
            ConversionError::Cancelled => {}
            ConversionError::FileWriteError { file_name, detail } => {
                params.insert("fileName".to_string(), file_name.clone());
                params.insert("detail".to_string(), detail.clone());
            }
            ConversionError::DirectoryCreateError { path, detail } => {
                params.insert("path".to_string(), path.clone());
                params.insert("detail".to_string(), detail.clone());
            }
        }
        params
    }
}

/// バイト数を人間が読みやすい形式にフォーマットする
fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1_024 {
        format!("{:.1}KB", bytes as f64 / 1_024.0)
    } else {
        format!("{}B", bytes)
    }
}

// Make ConversionError serializable for Tauri IPC
impl From<ConversionError> for String {
    fn from(error: ConversionError) -> Self {
        error.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_key_mapping() {
        let err = ConversionError::DiskSpaceError {
            required_bytes: 1_000_000,
            available_bytes: 500_000,
        };
        assert_eq!(err.error_key(), "errors.diskSpace");

        let err = ConversionError::PermissionError {
            path: "/tmp/test".to_string(),
        };
        assert_eq!(err.error_key(), "errors.permission");

        let err = ConversionError::Cancelled;
        assert_eq!(err.error_key(), "errors.cancelled");
    }

    #[test]
    fn test_error_params_disk_space() {
        let err = ConversionError::DiskSpaceError {
            required_bytes: 2_097_152, // 2MB
            available_bytes: 1_048_576, // 1MB
        };
        let params = err.error_params();
        assert_eq!(params.get("required").unwrap(), "2.0MB");
        assert_eq!(params.get("available").unwrap(), "1.0MB");
    }

    #[test]
    fn test_error_params_parse_error() {
        let err = ConversionError::ParseError {
            file_name: "test.json".to_string(),
            line: Some(42),
            detail: "unexpected token".to_string(),
        };
        let params = err.error_params();
        assert_eq!(params.get("fileName").unwrap(), "test.json");
        assert_eq!(params.get("line").unwrap(), "42");
        assert_eq!(params.get("detail").unwrap(), "unexpected token");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500B");
        assert_eq!(format_bytes(1_024), "1.0KB");
        assert_eq!(format_bytes(1_048_576), "1.0MB");
        assert_eq!(format_bytes(1_073_741_824), "1.0GB");
    }
}
