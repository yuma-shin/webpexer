# Implementation Plan: Universal File Converter

## Overview

既存の WebPexer（Electron ベース）を Tauri 2.x + React + TailwindCSS + HeadlessUI で1から再開発する。バックエンド（Rust）で画像・テキスト変換処理を実行し、フロントエンド（TypeScript/React）で直感的なUIを提供する汎用ファイル変換デスクトップアプリを構築する。

## Tasks

- [x] 1. プロジェクト初期セットアップとコアインターフェース定義
  - [x] 1.1 Tauri 2.x プロジェクトの作成と基本構成
    - `create-tauri-app` で新規プロジェクトを作成（React + TypeScript テンプレート）
    - `src-tauri/Cargo.toml` に必要な依存関係を追加（`image`, `serde_json`, `serde_yaml`, `toml`, `quick-xml`, `csv`, `encoding_rs`, `chardetng`, `tokio`, `thiserror`, `chrono`, `uuid`, `proptest`(dev)）
    - フロントエンド `package.json` に依存関係を追加（`tailwindcss`, `@headlessui/react`, `i18next`, `react-i18next`, `@tauri-apps/api`, `@tauri-apps/plugin-store`）
    - TailwindCSS と PostCSS の設定ファイルを作成
    - Vite 設定を確認・調整
    - _Requirements: 1.1, 1.2, 1.3, 1.4_

  - [x] 1.2 Rust バックエンドのデータモデルと型定義を作成
    - `src-tauri/src/models.rs` に `FileFormat`, `ConversionParams`, `ConversionOptions`, `ResizeOptions`, `TextEncoding`, `CsvDelimiter`, `FileNamingPattern`, `ConflictResolution` 列挙型・構造体を定義
    - `src-tauri/src/models.rs` に `ProgressEvent`, `ProgressStatus`, `ConversionResult`, `FailedFileInfo` を定義
    - `src-tauri/src/errors.rs` に `ConversionError` 列挙型を `thiserror` で定義
    - `src-tauri/src/history.rs` に `HistoryEntry`, `HistoryStatus`, `HistoryStore` を定義
    - `src-tauri/src/settings.rs` に `AppSettings`, `ThemeMode` を定義
    - _Requirements: 2.1, 3.1, 5.3, 5.4, 8.1_

  - [x] 1.3 フロントエンドの型定義と Tauri コマンドラッパーを作成
    - `src/types/index.ts` に共通型定義（`ImageFormat`, `TextFormat`, `FileFormat`, `ConversionParams`, `ConversionOptions`, `ProgressEvent`, `ConversionResult` 等）
    - `src/lib/tauri-commands.ts` に Tauri invoke ラッパー関数群を作成
    - _Requirements: 1.4, 2.1, 3.1_

  - [x] 1.4 Converter Trait とエンジン骨格を実装
    - `src-tauri/src/converter/mod.rs` に `Converter` trait を定義（`supported_input_formats`, `supported_output_formats`, `can_convert`, `convert`）
    - `src-tauri/src/converter/mod.rs` に `ConversionEngine` 構造体とキャンセルトークン機構を実装
    - `ImageConverter` と `TextConverter` のスタブ実装を作成
    - _Requirements: 1.4, 2.1, 3.1_

- [x] 2. 入力バリデーションとファイル操作基盤
  - [x] 2.1 Validator モジュールを実装
    - `src-tauri/src/validator.rs` に `Validator` 構造体を作成
    - `validate_input_files`: ファイル存在確認、フォーマット判定
    - `validate_output_path`: 出力先の書き込み権限確認
    - `check_disk_space`: 入力ファイル合計サイズと出力先空き容量の比較
    - `validate_file_size`: テキストファイルの 50MB 上限チェック
    - _Requirements: 3.7, 5.6, 5.8, 10.2, 10.4_

  - [x] 2.2 Validator のプロパティテストを作成
    - **Property 14: ディスク容量事前検証**
    - **Validates: Requirements 10.4**

  - [x] 2.3 出力パス生成とファイル命名ロジックを実装
    - `src-tauri/src/output.rs` にデフォルト出力パス生成ロジック（`{source_dir}/{format_name}/{filename}.{ext}`）
    - ファイル命名パターン実装: `original`, `original_sequential`（ゼロ埋め3桁）, `original_datetime`（yyyyMMdd_HHmmss）
    - ファイル衝突解決実装: `overwrite`, `skip`, `rename`（接尾辞 `_N`, N≥2）
    - _Requirements: 5.2, 5.3, 5.4_

  - [x] 2.4 出力パス生成のプロパティテストを作成
    - **Property 10: デフォルト出力パス生成**
    - **Property 11: ファイル命名パターンの正確性**
    - **Property 12: ファイル衝突解決の正確性**
    - **Validates: Requirements 5.2, 5.3, 5.4**

- [x] 3. 画像変換エンジンの実装
  - [x] 3.1 ImageConverter の変換ロジックを実装
    - `src-tauri/src/converter/image.rs` に `ImageConverter` を実装
    - `image` crate を使用して 8 フォーマット間（PNG, JPEG, WebP, GIF, BMP, TIFF, AVIF, ICO）の相互変換を実装
    - 品質設定の適用（非可逆: JPEG, WebP, AVIF のみ、可逆フォーマットでは無視）
    - リサイズ処理（幅/高さ指定、アスペクト比維持計算）
    - _Requirements: 2.1, 2.3, 2.4, 2.6_

  - [x] 3.2 画像変換のプロパティテストを作成
    - **Property 1: 画像フォーマット変換の整合性**
    - **Property 2: リサイズ処理の寸法正確性**
    - **Property 3: 可逆フォーマットにおける品質設定の無視**
    - **Validates: Requirements 2.1, 2.4, 2.6**

  - [x] 3.3 画像変換のユニットテストを作成
    - 各フォーマットペアの基本変換テスト（PNG→JPEG, WebP→PNG 等、代表的な組み合わせ）
    - エッジケーステスト: 最小画像（1x1）、大画像（16383x16383リサイズ指定）
    - 品質値境界テスト: 1, 50, 100
    - _Requirements: 2.1, 2.3, 2.4, 2.6_

- [x] 4. テキスト変換エンジンの実装
  - [x] 4.1 TextConverter の構造化データ変換を実装
    - `src-tauri/src/converter/text.rs` に `TextConverter` を実装
    - JSON↔YAML 変換（`serde_json` + `serde_yaml`）
    - JSON↔TOML 変換（`serde_json` + `toml`）
    - JSON↔XML 変換（`serde_json` + `quick-xml`）
    - CSV→JSON 変換（`csv` crate、区切り文字オプション: カンマ/タブ/セミコロン）
    - Markdown↔PlainText 変換
    - ネスト構造の CSV 変換拒否チェック
    - _Requirements: 3.1, 3.2, 3.3, 3.8_

  - [x] 4.2 エンコーディング検出と変換を実装
    - `encoding_rs` と `chardetng` を使用した入力ファイルのエンコーディング自動検出
    - 出力エンコーディング変換（UTF-8, Shift_JIS, EUC-JP）
    - _Requirements: 3.4_

  - [x] 4.3 テキスト変換のプロパティテストを作成
    - **Property 4: JSON↔YAML ラウンドトリップ保存**
    - **Property 5: CSV→JSON 変換の構造保存**
    - **Property 6: ネスト構造の CSV 変換拒否**
    - **Property 20: エンコーディング変換の正確性**
    - **Validates: Requirements 3.2, 3.3, 3.4, 3.6, 3.8**

  - [x] 4.4 テキスト変換のユニットテストを作成
    - 各変換ペアの基本テスト
    - パースエラー時の行番号付きエラーメッセージ検証
    - 50MB ファイルサイズ上限テスト
    - CSV 区切り文字オプションテスト
    - _Requirements: 3.1, 3.2, 3.3, 3.5, 3.7_

- [x] 5. チェックポイント - バックエンド変換エンジン検証
  - Ensure all tests pass, ask the user if questions arise.

- [x] 6. バッチ処理とTauriコマンド統合
  - [x] 6.1 バッチ変換処理と進捗通知を実装
    - `src-tauri/src/commands/mod.rs` に `start_conversion` コマンドを実装
    - Tauri Channel を使用した進捗イベント送信（ファイルごとの処理状況）
    - `CancellationToken` によるキャンセル処理実装
    - ファイル単位のエラーハンドリング（スキップ＆継続）
    - 不完全出力ファイルのクリーンアップ処理
    - ソースファイル削除ロジック（変換成功ファイルのみ削除）
    - _Requirements: 4.4, 4.5, 4.6, 4.7, 5.5, 5.7, 10.3, 10.5, 10.6_

  - [x] 6.2 補助 Tauri コマンドを実装
    - `cancel_conversion`: キャンセルトークン発行
    - `select_folder`: Tauri ネイティブダイアログでフォルダ選択
    - `get_supported_formats`: カテゴリ別サポートフォーマット一覧取得
    - `validate_output_path`: 出力先パス検証
    - `check_disk_space`: ディスク容量事前確認
    - _Requirements: 4.6, 5.1, 5.6, 10.4_

  - [x] 6.3 バッチ処理のプロパティテストを作成
    - **Property 7: バッチ処理のフォールトトレランス**
    - **Property 8: 進捗報告の正確性**
    - **Property 9: キャンセル後の処理停止**
    - **Property 13: 条件付きソースファイル削除**
    - **Property 15: 失敗時の不完全出力クリーンアップ**
    - **Validates: Requirements 2.5, 4.4, 4.5, 4.6, 4.7, 5.5, 10.1, 10.5, 10.6**

- [x] 7. 国際化とテーマシステム
  - [x] 7.1 i18next セットアップとロケールファイル作成
    - `src/lib/i18n.ts` に i18next + react-i18next 設定を作成
    - `src/locales/en.json` と `src/locales/jp.json` に翻訳キーを定義
    - OS ロケール検出ロジック（`ja`/`jp` → 日本語、`en` → 英語、その他 → 英語フォールバック）
    - ユーザー設定永続化との統合（Tauri Store Plugin 使用）
    - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_

  - [x] 7.2 テーマシステム（ダーク/ライトモード）を実装
    - `src/lib/theme.ts` にテーマ管理ロジックを作成
    - OS カラースキーム検出と初期値設定
    - TailwindCSS dark mode クラス切替の実装
    - テーマ設定の永続化（Tauri Store Plugin）
    - _Requirements: 6.2, 6.8_

  - [x] 7.3 国際化のプロパティテストを作成
    - **Property 18: 言語選択ロジック**
    - **Property 19: 翻訳キーの完全性**
    - **Validates: Requirements 7.2, 7.3, 7.5**

- [x] 8. フロントエンド UI コンポーネント実装
  - [x] 8.1 レイアウトとヘッダーコンポーネント
    - `src/App.tsx` にルートレイアウトを作成
    - `src/components/layout/Header.tsx`: ダークモードトグル、言語セレクタ
    - `src/components/layout/NotificationArea.tsx`: 成功/エラー通知表示
    - TailwindCSS ベースのミニマルデザイン（白背景、余白16px+、角丸4px+、シャドウ）
    - 最小ウィンドウサイズ設定（幅400px、高さ300px）
    - _Requirements: 6.1, 6.2, 6.5, 6.6, 6.7, 6.8_

  - [x] 8.2 ファイル選択コンポーネント
    - `src/components/file/DropZone.tsx`: ドラッグ＆ドロップエリア（ホバー時ハイライト）
    - `src/components/file/FileList.tsx`: 選択ファイルリスト表示
    - ドロップゾーンのインタラクション実装（境界線色・背景色変化）
    - ネイティブファイル/フォルダダイアログ呼び出し連携
    - _Requirements: 4.1, 4.2, 4.3, 6.3, 6.4_

  - [x] 8.3 変換設定コンポーネント
    - `src/components/conversion/FormatSelector.tsx`: カテゴリ別フォーマット選択（HeadlessUI Listbox）
    - `src/components/conversion/QualitySlider.tsx`: 品質スライダー（1-100）
    - `src/components/conversion/ResizeOptions.tsx`: リサイズオプション（幅/高さ/アスペクト比維持）
    - `src/components/conversion/OutputSettings.tsx`: 出力先設定、命名パターン、衝突解決、ソース削除オプション
    - `src/components/conversion/ConvertButton.tsx`: 変換実行ボタン
    - _Requirements: 2.3, 2.4, 5.1, 5.2, 5.3, 5.4, 5.5_

  - [x] 8.4 進捗表示コンポーネント
    - `src/components/progress/ProgressBar.tsx`: プログレスバー（処理済み/総数）
    - `src/components/progress/ProgressDetail.tsx`: 現在処理中のファイル名表示
    - キャンセルボタンの統合
    - _Requirements: 4.5, 4.6_

- [x] 9. チェックポイント - フロントエンド基盤検証
  - Ensure all tests pass, ask the user if questions arise.

- [x] 10. カスタムフックと状態管理
  - [x] 10.1 変換処理フックを実装
    - `src/hooks/useConversion.ts`: 変換開始、キャンセル、進捗監視、結果処理
    - Tauri Channel イベントリスナーの購読と解除
    - 変換状態管理（idle, converting, completed, cancelled, error）
    - _Requirements: 4.4, 4.5, 4.6, 4.7_

  - [x] 10.2 ファイル選択フックを実装
    - `src/hooks/useFileSelection.ts`: ドラッグ＆ドロップハンドリング、ファイルダイアログ連携、ファイルリスト管理
    - フォルダ選択時のサブフォルダ再帰探索（バックエンド側で処理）
    - _Requirements: 4.1, 4.2, 4.3, 4.8_

  - [x] 10.3 設定管理フックを実装
    - `src/hooks/useSettings.ts`: Tauri Store Plugin による設定読み書き
    - デフォルト設定の提供と設定変更の永続化
    - _Requirements: 5.3, 5.4, 5.5, 7.2_

- [x] 11. 変換履歴機能
  - [x] 11.1 履歴管理バックエンドを実装
    - `src-tauri/src/commands/history.rs` に `get_conversion_history`, `clear_conversion_history` コマンドを実装
    - 変換完了時の自動履歴記録（最大50件、FIFO）
    - Tauri Store Plugin による永続化
    - _Requirements: 8.1, 8.2, 8.5_

  - [x] 11.2 履歴 UI コンポーネントを実装
    - `src/components/history/HistoryPanel.tsx`: 履歴一覧表示（降順、最大50件）
    - `src/components/history/HistoryItem.tsx`: 個別履歴項目（フォルダパス、フォーマット、日時、ステータス）
    - `src/hooks/useHistory.ts`: 履歴取得、設定復元、クリア機能
    - 履歴選択時の設定復元ロジック（存在しないフォルダパスの検知と通知）
    - _Requirements: 8.2, 8.3, 8.4, 8.5_

  - [x] 11.3 履歴管理のプロパティテストを作成
    - **Property 16: 履歴件数上限の維持**
    - **Property 17: 履歴復元の設定一致**
    - **Validates: Requirements 8.2, 8.3**

- [x] 12. メインビューの統合と全体ワイヤリング
  - [x] 12.1 メインビューページを統合
    - `src/pages/MainView.tsx` に全コンポーネントを配置・統合
    - DropZone → FormatSelector → ConversionOptions → ConvertButton のフロー接続
    - 進捗表示と通知エリアの連携
    - 履歴パネルの統合
    - _Requirements: 6.3, 4.4, 4.5_

  - [x] 12.2 100MB超ファイルの確認ダイアログを実装
    - バックエンド: ファイルサイズチェックとダイアログトリガー
    - フロントエンド: 確認ダイアログ UI（ファイルサイズ表示、続行/スキップ選択）
    - _Requirements: 9.4, 9.5, 9.6_

  - [x] 12.3 Tauri ウィンドウ設定とビルド設定
    - `src-tauri/tauri.conf.json` にウィンドウ設定（最小サイズ 400x300、タイトル、アイコン）
    - パーミッション設定（ファイルシステムアクセス、ダイアログ、ストア）
    - `electron-builder.yml` を削除し Tauri ビルド設定に置き換え
    - _Requirements: 1.1, 1.5, 6.7_

- [x] 13. チェックポイント - 全体統合検証
  - Ensure all tests pass, ask the user if questions arise.

- [x] 14. パフォーマンス最適化とエラーハンドリング仕上げ
  - [x] 14.1 バックグラウンド処理とメモリ管理を最適化
    - 変換処理の非同期実行確認（tokio::spawn）
    - メモリ使用量の制御（大画像のストリーミング処理検討）
    - UI レスポンス維持の確認（200ms 以内の応答）
    - _Requirements: 9.1, 9.2, 9.3_

  - [x] 14.2 エラーハンドリングとエラー表示を仕上げ
    - バックエンド: 全エラーパスの i18n キーマッピング
    - フロントエンド: エラーメッセージの翻訳表示
    - 起動時バックエンド初期化失敗時のエラーメッセージ表示
    - _Requirements: 1.7, 2.5, 3.5, 10.1, 10.2, 10.3_

  - [x] 14.3 統合テストを作成
    - Tauri コマンド呼び出し → Rust 処理 → 結果返却の一連フロー検証
    - ファイルシステム操作テスト（フォルダ作成、書き込み、削除）
    - 設定・履歴の永続化テスト
    - _Requirements: 1.4, 8.1, 5.2_

- [x] 15. 最終チェックポイント - 全テスト実行と最終確認
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- タスクに `*` が付いているものはオプションであり、MVP 構築を優先する場合はスキップ可能
- 各タスクは具体的な Requirements への参照を含み、トレーサビリティを確保
- チェックポイントでインクリメンタルな検証を実施
- プロパティテストは `proptest` crate（Rust）を使用して実装
- フロントエンドのユニットテストは Vitest を使用
- Tauri 2.x の新しい Channel API を進捗通知に使用（Event API の代わり）

## Task Dependency Graph

```json
{
  "waves": [
    { "id": 0, "tasks": ["1.1"] },
    { "id": 1, "tasks": ["1.2", "1.3"] },
    { "id": 2, "tasks": ["1.4", "2.1"] },
    { "id": 3, "tasks": ["2.2", "2.3", "3.1"] },
    { "id": 4, "tasks": ["2.4", "3.2", "3.3", "4.1"] },
    { "id": 5, "tasks": ["4.2", "4.3", "4.4"] },
    { "id": 6, "tasks": ["6.1", "7.1", "7.2"] },
    { "id": 7, "tasks": ["6.2", "6.3", "7.3"] },
    { "id": 8, "tasks": ["8.1", "8.2"] },
    { "id": 9, "tasks": ["8.3", "8.4"] },
    { "id": 10, "tasks": ["10.1", "10.2", "10.3"] },
    { "id": 11, "tasks": ["11.1"] },
    { "id": 12, "tasks": ["11.2", "11.3"] },
    { "id": 13, "tasks": ["12.1", "12.2", "12.3"] },
    { "id": 14, "tasks": ["14.1", "14.2"] },
    { "id": 15, "tasks": ["14.3"] }
  ]
}
```
