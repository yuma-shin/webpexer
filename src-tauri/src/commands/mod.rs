use std::path::{Path, PathBuf};
use std::sync::Mutex;

use chrono::Utc;
use tauri::ipc::Channel;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use uuid::Uuid;

use crate::converter::image::ImageConverter;
use crate::converter::text::TextConverter;
use crate::converter::{ConversionEngine, ConverterBoxed};
use crate::history::{HistoryEntry, HistoryState, HistoryStatus};
use crate::models::{
    ConversionParams, ConversionResult, DiskSpaceCheck, FailedFileInfo, FileFormat, FormatCategory,
    FormatInfo, LargeFileInfo, PathValidation, ProgressEvent, ProgressStatus,
};
use crate::output::{ensure_output_dir, generate_output_path, OutputPathResult};
use crate::validator::Validator;

/// バッチ変換のState管理
pub struct ConversionState {
    pub engine: Mutex<ConversionEngine>,
}

impl ConversionState {
    pub fn new() -> Self {
        Self {
            engine: Mutex::new(ConversionEngine::new()),
        }
    }
}

impl Default for ConversionState {
    fn default() -> Self {
        Self::new()
    }
}

/// 変換を開始する
///
/// Tauri の async コマンドとして実行されるため、Tokio 非同期ランタイム上で動作する。
/// 各ファイルの変換処理は以下のように最適化されている:
///
/// - **画像変換**: CPU バウンドな処理（デコード・リサイズ・エンコード）は
///   `tokio::task::spawn_blocking` で専用スレッドプールに委譲し、
///   async ランタイムのワーカースレッドをブロックしない。
/// - **テキスト変換**: 比較的軽量なため async コンテキスト内で直接実行する。
/// - **進捗通知**: Tauri Channel による非同期イベント送信で UI レスポンスを維持。
/// - **キャンセル**: ファイル間のループで `CancellationToken` をチェックし、
///   即座に処理を中断可能。
#[tauri::command]
pub async fn start_conversion(
    state: State<'_, ConversionState>,
    history_state: State<'_, HistoryState>,
    params: ConversionParams,
    channel: Channel<ProgressEvent>,
) -> Result<ConversionResult, String> {
    // a. キャンセルトークンのリセット
    {
        let mut engine = state.engine.lock().map_err(|e| e.to_string())?;
        engine.reset_cancel();
    }

    // b. 出力パスの検証（出力先が指定されている場合、作成を試みる）
    let output_dir = if let Some(ref dir) = params.output_dir {
        Validator::validate_output_path(dir).map_err(|e| e.to_string())?;
        Some(dir.clone())
    } else {
        None
    };

    // c. 入力ファイルの合計サイズを計算し、ディスク容量確認
    let total_input_size: u64 = params
        .input_paths
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    // ディスク容量チェック（出力先が指定されている場合）
    if let Some(ref dir) = output_dir {
        Validator::check_disk_space(total_input_size, dir).map_err(|e| e.to_string())?;
    }

    // d. 入力ファイルのバリデーションとフォーマット検出
    let validation = Validator::validate_input_files(&params.input_paths);

    let valid_files = validation.valid_files;
    let total_count = valid_files.len() as u32;

    // 結果の初期化
    let mut success_count: u32 = 0;
    let mut failed_count: u32 = 0;
    let mut skipped_count: u32 = 0;
    let mut failed_files: Vec<FailedFileInfo> = Vec::new();

    // 無効ファイルを失敗として記録
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

    // 出力ディレクトリの決定（結果に含めるため）
    let result_output_dir = output_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));

    // e. 各ファイルの変換処理
    for (idx, (file_path, input_format)) in valid_files.iter().enumerate() {
        // キャンセルチェック
        let is_cancelled = {
            let engine = state.engine.lock().map_err(|e| e.to_string())?;
            engine.is_cancelled()
        };

        if is_cancelled {
            // キャンセルされた場合、残りのファイルについてはキャンセルとして進捗を送信
            let file_name = file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let _ = channel.send(ProgressEvent {
                current_file: file_name,
                processed_count: (idx as u32) + 1,
                total_count,
                status: ProgressStatus::Cancelled,
                error: None,
            });
            break;
        }

        let file_name = file_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // 進捗: 処理中を通知
        let _ = channel.send(ProgressEvent {
            current_file: file_name.clone(),
            processed_count: idx as u32,
            total_count,
            status: ProgressStatus::Processing,
            error: None,
        });

        // 変換ペアのサポート確認
        let can_convert = {
            let engine = state.engine.lock().map_err(|e| e.to_string())?;
            engine.can_convert(input_format, &params.output_format)
        };

        if !can_convert {
            // 非対応変換ペア: スキップとして扱う
            let error_msg = format!(
                "非対応変換ペア: {:?} → {:?}",
                input_format, params.output_format
            );
            failed_files.push(FailedFileInfo {
                file_name: file_name.clone(),
                error: error_msg.clone(),
            });
            failed_count += 1;
            let _ = channel.send(ProgressEvent {
                current_file: file_name,
                processed_count: (idx as u32) + 1,
                total_count,
                status: ProgressStatus::Failed,
                error: Some(error_msg),
            });
            continue;
        }

        // 出力パスの生成
        let output_path_result = generate_output_path(
            file_path,
            output_dir.as_deref(),
            &params.output_format,
            &params.options.file_naming,
            &params.options.conflict_resolution,
            (idx as u32) + 1,
        );

        let output_file_path = match output_path_result {
            OutputPathResult::Write(path) => path,
            OutputPathResult::Skip => {
                // スキップ（ファイルが既に存在し、Skip モードが選択されている場合）
                skipped_count += 1;
                let _ = channel.send(ProgressEvent {
                    current_file: file_name,
                    processed_count: (idx as u32) + 1,
                    total_count,
                    status: ProgressStatus::Completed,
                    error: None,
                });
                continue;
            }
        };

        // 出力ディレクトリが存在しない場合は作成
        if let Err(e) = ensure_output_dir(file_path, output_dir.as_deref(), &params.output_format) {
            failed_files.push(FailedFileInfo {
                file_name: file_name.clone(),
                error: e.clone(),
            });
            failed_count += 1;
            let _ = channel.send(ProgressEvent {
                current_file: file_name,
                processed_count: (idx as u32) + 1,
                total_count,
                status: ProgressStatus::Failed,
                error: Some(e),
            });
            continue;
        }

        // 変換実行
        // コンバータを直接作成して使用（Mutexロックのライフタイム問題を回避）
        let converter: Box<dyn ConverterBoxed> = if is_image_format(input_format)
            && is_image_format(&params.output_format)
        {
            Box::new(ImageConverter::new())
        } else {
            Box::new(TextConverter::new())
        };
        let convert_result = converter
            .convert_boxed(file_path, &output_file_path, &params.options)
            .await;

        match convert_result {
            Ok(()) => {
                success_count += 1;
                let _ = channel.send(ProgressEvent {
                    current_file: file_name.clone(),
                    processed_count: (idx as u32) + 1,
                    total_count,
                    status: ProgressStatus::Completed,
                    error: None,
                });

                // ソースファイル削除（変換成功かつオプション有効の場合のみ）
                if params.options.delete_source {
                    if let Err(_e) = std::fs::remove_file(file_path) {
                        // 削除失敗は警告として記録するがバッチ処理は継続
                        // ログ出力のみ（結果を失敗にはしない）
                    }
                }
            }
            Err(e) => {
                // 失敗時: 不完全な出力ファイルをクリーンアップ
                if output_file_path.exists() {
                    let _ = std::fs::remove_file(&output_file_path);
                }

                let error_msg = e.to_string();
                failed_files.push(FailedFileInfo {
                    file_name: file_name.clone(),
                    error: error_msg.clone(),
                });
                failed_count += 1;

                let _ = channel.send(ProgressEvent {
                    current_file: file_name,
                    processed_count: (idx as u32) + 1,
                    total_count,
                    status: ProgressStatus::Failed,
                    error: Some(error_msg),
                });
            }
        }
    }

    // f. 履歴エントリの記録
    let source_folder = params
        .input_paths
        .first()
        .and_then(|p| p.parent())
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    let history_status = if failed_count > 0 && success_count == 0 {
        HistoryStatus::Failed
    } else {
        HistoryStatus::Success
    };

    let history_entry = HistoryEntry {
        id: Uuid::new_v4().to_string(),
        source_folder,
        output_format: params.output_format.clone(),
        cleanup_enabled: params.options.delete_source,
        executed_at: Utc::now(),
        status: history_status,
        file_count: success_count + failed_count + skipped_count,
    };

    if let Ok(mut store) = history_state.store.lock() {
        store.add_entry(history_entry);
    }

    // g. 結果を返す
    Ok(ConversionResult {
        success_count,
        failed_count,
        skipped_count,
        failed_files,
        output_dir: result_output_dir,
    })
}

/// 変換をキャンセルする
#[tauri::command]
pub async fn cancel_conversion(state: State<'_, ConversionState>) -> Result<(), String> {
    let engine = state.engine.lock().map_err(|e| e.to_string())?;
    engine.cancel();
    Ok(())
}

/// フォルダ選択ダイアログを表示
#[tauri::command]
pub async fn select_folder(app: AppHandle) -> Result<Option<String>, String> {
    let folder_path = app.dialog().file().blocking_pick_folder();

    match folder_path {
        Some(file_path) => {
            // FilePath を文字列に変換
            Ok(Some(file_path.to_string()))
        }
        None => Ok(None),
    }
}

/// サポートフォーマット一覧を取得
#[tauri::command]
pub async fn get_supported_formats(category: FormatCategory) -> Vec<FormatInfo> {
    match category {
        FormatCategory::Image => vec![
            FormatInfo {
                format: crate::models::FileFormat::Png,
                name: "PNG".to_string(),
                extensions: vec!["png".to_string()],
                category: FormatCategory::Image,
            },
            FormatInfo {
                format: crate::models::FileFormat::Jpeg,
                name: "JPEG".to_string(),
                extensions: vec!["jpg".to_string(), "jpeg".to_string()],
                category: FormatCategory::Image,
            },
            FormatInfo {
                format: crate::models::FileFormat::Webp,
                name: "WebP".to_string(),
                extensions: vec!["webp".to_string()],
                category: FormatCategory::Image,
            },
            FormatInfo {
                format: crate::models::FileFormat::Gif,
                name: "GIF".to_string(),
                extensions: vec!["gif".to_string()],
                category: FormatCategory::Image,
            },
            FormatInfo {
                format: crate::models::FileFormat::Bmp,
                name: "BMP".to_string(),
                extensions: vec!["bmp".to_string()],
                category: FormatCategory::Image,
            },
            FormatInfo {
                format: crate::models::FileFormat::Tiff,
                name: "TIFF".to_string(),
                extensions: vec!["tiff".to_string(), "tif".to_string()],
                category: FormatCategory::Image,
            },
            FormatInfo {
                format: crate::models::FileFormat::Avif,
                name: "AVIF".to_string(),
                extensions: vec!["avif".to_string()],
                category: FormatCategory::Image,
            },
            FormatInfo {
                format: crate::models::FileFormat::Ico,
                name: "ICO".to_string(),
                extensions: vec!["ico".to_string()],
                category: FormatCategory::Image,
            },
        ],
        FormatCategory::Text => vec![
            FormatInfo {
                format: crate::models::FileFormat::Json,
                name: "JSON".to_string(),
                extensions: vec!["json".to_string()],
                category: FormatCategory::Text,
            },
            FormatInfo {
                format: crate::models::FileFormat::Yaml,
                name: "YAML".to_string(),
                extensions: vec!["yaml".to_string(), "yml".to_string()],
                category: FormatCategory::Text,
            },
            FormatInfo {
                format: crate::models::FileFormat::Toml,
                name: "TOML".to_string(),
                extensions: vec!["toml".to_string()],
                category: FormatCategory::Text,
            },
            FormatInfo {
                format: crate::models::FileFormat::Xml,
                name: "XML".to_string(),
                extensions: vec!["xml".to_string()],
                category: FormatCategory::Text,
            },
            FormatInfo {
                format: crate::models::FileFormat::Csv,
                name: "CSV".to_string(),
                extensions: vec!["csv".to_string()],
                category: FormatCategory::Text,
            },
            FormatInfo {
                format: crate::models::FileFormat::Markdown,
                name: "Markdown".to_string(),
                extensions: vec!["md".to_string(), "markdown".to_string()],
                category: FormatCategory::Text,
            },
            FormatInfo {
                format: crate::models::FileFormat::PlainText,
                name: "Plain Text".to_string(),
                extensions: vec!["txt".to_string()],
                category: FormatCategory::Text,
            },
        ],
    }
}

/// 出力パスの検証
#[tauri::command]
pub async fn validate_output_path(path: String) -> Result<PathValidation, String> {
    let path_buf = Path::new(&path);

    match Validator::validate_output_path(path_buf) {
        Ok(()) => Ok(PathValidation {
            is_valid: true,
            is_writable: true,
            error: None,
        }),
        Err(e) => {
            // PermissionError の場合は is_writable = false
            let is_permission_error = matches!(
                e,
                crate::errors::ConversionError::PermissionError { .. }
            );
            Ok(PathValidation {
                is_valid: !is_permission_error,
                is_writable: false,
                error: Some(e.to_string()),
            })
        }
    }
}

/// ディスク容量チェック
#[tauri::command]
pub async fn check_disk_space(
    input_paths: Vec<String>,
    output_path: String,
) -> Result<DiskSpaceCheck, String> {
    let output_dir = Path::new(&output_path);

    // 入力ファイルの合計サイズを計算
    let required_bytes: u64 = input_paths
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    // 出力先の空き容量を取得
    let available_bytes = Validator::get_available_space(output_dir);

    let has_enough_space = required_bytes <= available_bytes;

    Ok(DiskSpaceCheck {
        has_enough_space,
        required_bytes,
        available_bytes,
    })
}

/// 変換履歴を取得
#[tauri::command]
pub async fn get_conversion_history(
    history_state: State<'_, HistoryState>,
) -> Result<Vec<HistoryEntry>, String> {
    let store = history_state.store.lock().map_err(|e| e.to_string())?;
    Ok(store.get_all().to_vec())
}

/// 変換履歴をクリア
#[tauri::command]
pub async fn clear_conversion_history(
    history_state: State<'_, HistoryState>,
) -> Result<(), String> {
    let mut store = history_state.store.lock().map_err(|e| e.to_string())?;
    store.clear();
    Ok(())
}


/// 指定閾値を超えるファイルの一覧を返す（100MB超ファイル確認用）
#[tauri::command]
pub async fn check_large_files(
    input_paths: Vec<String>,
    threshold_mb: Option<u64>,
) -> Result<Vec<LargeFileInfo>, String> {
    let threshold = threshold_mb.unwrap_or(100);
    let threshold_bytes = threshold * 1024 * 1024;

    let mut large_files: Vec<LargeFileInfo> = Vec::new();

    for path_str in &input_paths {
        let path = Path::new(path_str);
        match std::fs::metadata(path) {
            Ok(metadata) => {
                let file_size = metadata.len();
                if file_size > threshold_bytes {
                    let name = path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path_str.clone());
                    let size_mb = file_size as f64 / (1024.0 * 1024.0);
                    large_files.push(LargeFileInfo {
                        name,
                        size_mb: (size_mb * 10.0).round() / 10.0, // 小数第1位まで
                    });
                }
            }
            Err(_) => {
                // メタデータ取得失敗はスキップ（変換時に別途エラーハンドリング）
            }
        }
    }

    Ok(large_files)
}

/// フォーマットが画像フォーマットかどうかを判定するヘルパー
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
