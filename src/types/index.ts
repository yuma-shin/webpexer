// ============================================================
// 共通型定義 - Rust バックエンドモデルに対応
// ============================================================

// --- File Formats ---

/** 画像フォーマット (serde: rename_all = "lowercase") */
export type ImageFormat =
  | "png"
  | "jpeg"
  | "webp"
  | "gif"
  | "bmp"
  | "tiff"
  | "avif"
  | "ico";

/** テキストフォーマット (serde: rename_all = "lowercase") */
export type TextFormat =
  | "json"
  | "yaml"
  | "toml"
  | "xml"
  | "csv"
  | "markdown"
  | "plaintext";

/** 全ファイルフォーマット */
export type FileFormat = ImageFormat | TextFormat;

/** フォーマットカテゴリ (serde: rename_all = "lowercase") */
export type FormatCategory = "image" | "text";

// --- Conversion Options ---

/** リサイズオプション */
export interface ResizeOptions {
  width?: number | null;
  height?: number | null;
  maintainAspectRatio: boolean;
}

/** テキストエンコーディング (serde: rename_all = "snake_case") */
export type TextEncoding = "utf8" | "shift_jis" | "euc_jp";

/** CSV区切り文字 (serde: rename_all = "snake_case") */
export type CsvDelimiter = "comma" | "tab" | "semicolon";

/** ファイル命名パターン (serde: rename_all = "snake_case") */
export type FileNamingPattern =
  | "original"
  | "original_sequential"
  | "original_datetime";

/** 衝突解決方式 (serde: rename_all = "snake_case") */
export type ConflictResolution = "overwrite" | "skip" | "rename";

/** 変換オプション */
export interface ConversionOptions {
  quality: number;
  resize?: ResizeOptions | null;
  encoding?: TextEncoding | null;
  csvDelimiter?: CsvDelimiter | null;
  fileNaming: FileNamingPattern;
  conflictResolution: ConflictResolution;
  deleteSource: boolean;
}

/** 変換パラメータ（バックエンドに送信） */
export interface ConversionParams {
  inputPaths: string[];
  outputFormat: FileFormat;
  outputDir?: string | null;
  options: ConversionOptions;
}

// --- Progress & Results ---

/** 進捗ステータス (serde: rename_all = "lowercase") */
export type ProgressStatus =
  | "processing"
  | "completed"
  | "failed"
  | "cancelled";

/** 進捗イベント（Channel 経由で受信） */
export interface ProgressEvent {
  currentFile: string;
  processedCount: number;
  totalCount: number;
  status: ProgressStatus;
  error?: string | null;
}

/** 変換結果 */
export interface ConversionResult {
  successCount: number;
  failedCount: number;
  skippedCount: number;
  failedFiles: FailedFileInfo[];
  outputDir: string;
}

/** 失敗ファイル情報 */
export interface FailedFileInfo {
  fileName: string;
  error: string;
}

// --- History ---

/** 履歴ステータス (serde: rename_all = "lowercase") */
export type HistoryStatus = "success" | "failed";

/** 変換履歴エントリ */
export interface HistoryEntry {
  id: string;
  sourceFolder: string;
  outputFormat: FileFormat;
  cleanupEnabled: boolean;
  executedAt: string; // ISO 8601 datetime string
  status: HistoryStatus;
  fileCount: number;
}

// --- Settings ---

/** テーマモード (serde: rename_all = "lowercase") */
export type ThemeMode = "system" | "light" | "dark";

/** アプリケーション設定 */
export interface AppSettings {
  language: string;
  theme: ThemeMode;
  defaultQuality: number;
  defaultOutputFormat: FileFormat;
  fileNaming: FileNamingPattern;
  conflictResolution: ConflictResolution;
  deleteSource: boolean;
}

// --- Format Information ---

/** フォーマット情報 */
export interface FormatInfo {
  format: FileFormat;
  name: string;
  extensions: string[];
  category: FormatCategory;
}

// --- Validation ---

/** パス検証結果 */
export interface PathValidation {
  isValid: boolean;
  isWritable: boolean;
  error?: string | null;
}

/** ディスク容量チェック結果 */
export interface DiskSpaceCheck {
  hasEnoughSpace: boolean;
  requiredBytes: number;
  availableBytes: number;
}

/** 大容量ファイル情報（100MB超ファイルの確認ダイアログ用） */
export interface LargeFileInfo {
  name: string;
  sizeMb: number;
}
