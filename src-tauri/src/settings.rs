use serde::{Deserialize, Serialize};

use crate::models::{ConflictResolution, FileFormat, FileNamingPattern};

/// アプリケーション設定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub language: String,
    pub theme: ThemeMode,
    pub default_quality: u8,
    pub default_output_format: FileFormat,
    pub file_naming: FileNamingPattern,
    pub conflict_resolution: ConflictResolution,
    pub delete_source: bool,
}

/// テーマモード
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: ThemeMode::System,
            default_quality: 80,
            default_output_format: FileFormat::Png,
            file_naming: FileNamingPattern::Original,
            conflict_resolution: ConflictResolution::Overwrite,
            delete_source: false,
        }
    }
}
