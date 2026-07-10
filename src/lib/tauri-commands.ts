import { invoke, Channel } from "@tauri-apps/api/core";
import type {
  ConversionParams,
  ConversionResult,
  DiskSpaceCheck,
  FormatCategory,
  FormatInfo,
  HistoryEntry,
  LargeFileInfo,
  PathValidation,
  ProgressEvent,
} from "../types";

/**
 * 変換を開始する
 *
 * Channel を通じて進捗イベントを受信し、変換完了後に結果を返す。
 */
export async function startConversion(
  params: ConversionParams,
  onProgress: (event: ProgressEvent) => void,
): Promise<ConversionResult> {
  const channel = new Channel<ProgressEvent>();
  channel.onmessage = onProgress;

  return await invoke<ConversionResult>("start_conversion", {
    params,
    channel,
  });
}

/**
 * 実行中の変換をキャンセルする
 */
export async function cancelConversion(): Promise<void> {
  await invoke<void>("cancel_conversion");
}

/**
 * ネイティブフォルダ選択ダイアログを表示する
 *
 * @returns 選択されたフォルダパス。キャンセル時は null。
 */
export async function selectFolder(): Promise<string | null> {
  return await invoke<string | null>("select_folder");
}

/**
 * カテゴリ別サポートフォーマット一覧を取得する
 */
export async function getSupportedFormats(
  category: FormatCategory,
): Promise<FormatInfo[]> {
  return await invoke<FormatInfo[]>("get_supported_formats", { category });
}

/**
 * 出力先パスを検証する
 */
export async function validateOutputPath(
  path: string,
): Promise<PathValidation> {
  return await invoke<PathValidation>("validate_output_path", { path });
}

/**
 * 入力ファイル群に対してディスク容量が十分かチェックする
 */
export async function checkDiskSpace(
  inputPaths: string[],
  outputPath: string,
): Promise<DiskSpaceCheck> {
  return await invoke<DiskSpaceCheck>("check_disk_space", {
    inputPaths,
    outputPath,
  });
}

/**
 * 変換履歴一覧を取得する（実行日時の降順、最大50件）
 */
export async function getConversionHistory(): Promise<HistoryEntry[]> {
  return await invoke<HistoryEntry[]>("get_conversion_history");
}

/**
 * 変換履歴を全件クリアする
 */
export async function clearConversionHistory(): Promise<void> {
  await invoke<void>("clear_conversion_history");
}

/**
 * 閾値を超える大容量ファイルの一覧を取得する
 *
 * @param inputPaths チェック対象のファイルパス一覧
 * @param thresholdMb 閾値（MB）。デフォルト100MB。
 * @returns 閾値を超えたファイルの情報リスト
 */
export async function checkLargeFiles(
  inputPaths: string[],
  thresholdMb?: number,
): Promise<LargeFileInfo[]> {
  return await invoke<LargeFileInfo[]>("check_large_files", {
    inputPaths,
    thresholdMb: thresholdMb ?? null,
  });
}
