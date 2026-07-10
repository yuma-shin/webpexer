pub mod image;
pub mod text;

#[cfg(test)]
mod image_tests;

#[cfg(test)]
mod text_tests;

use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::errors::ConversionError;
use crate::models::{ConversionOptions, FileFormat};

/// Converter trait - 各フォーマット変換器が実装する
///
/// `Send + Sync` が必要なため、非同期メソッドには RPITIT (return-position impl Trait in trait) を使用する。
/// ダイナミックディスパッチが必要な場合は `ConverterBoxed` を介して利用する。
pub trait Converter: Send + Sync {
    /// サポートする入力フォーマット一覧
    fn supported_input_formats(&self) -> &[FileFormat];

    /// サポートする出力フォーマット一覧
    fn supported_output_formats(&self) -> &[FileFormat];

    /// 指定されたフォーマットペアの変換が可能かどうかを判定
    fn can_convert(&self, from: &FileFormat, to: &FileFormat) -> bool;

    /// ファイル変換を実行
    ///
    /// # Arguments
    /// * `input` - 入力ファイルパス
    /// * `output` - 出力ファイルパス
    /// * `options` - 変換オプション（品質、リサイズ、エンコーディング等）
    fn convert(
        &self,
        input: &Path,
        output: &Path,
        options: &ConversionOptions,
    ) -> impl std::future::Future<Output = Result<(), ConversionError>> + Send;
}

/// Object-safe version of Converter trait for dynamic dispatch
///
/// `Converter` trait は RPITIT を使用しているためオブジェクトセーフではない。
/// `Box<dyn ConverterBoxed>` として `ConversionEngine` 内で使用する。
pub trait ConverterBoxed: Send + Sync {
    /// サポートする入力フォーマット一覧
    fn supported_input_formats(&self) -> &[FileFormat];

    /// サポートする出力フォーマット一覧
    fn supported_output_formats(&self) -> &[FileFormat];

    /// 指定されたフォーマットペアの変換が可能かどうかを判定
    fn can_convert(&self, from: &FileFormat, to: &FileFormat) -> bool;

    /// ファイル変換を実行（Box化された Future を返す）
    fn convert_boxed<'a>(
        &'a self,
        input: &'a Path,
        output: &'a Path,
        options: &'a ConversionOptions,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ConversionError>> + Send + 'a>,
    >;
}

/// Blanket implementation: `Converter` を実装するすべての型に対して `ConverterBoxed` を自動実装
impl<T: Converter> ConverterBoxed for T {
    fn supported_input_formats(&self) -> &[FileFormat] {
        Converter::supported_input_formats(self)
    }

    fn supported_output_formats(&self) -> &[FileFormat] {
        Converter::supported_output_formats(self)
    }

    fn can_convert(&self, from: &FileFormat, to: &FileFormat) -> bool {
        Converter::can_convert(self, from, to)
    }

    fn convert_boxed<'a>(
        &'a self,
        input: &'a Path,
        output: &'a Path,
        options: &'a ConversionOptions,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<(), ConversionError>> + Send + 'a>,
    > {
        Box::pin(Converter::convert(self, input, output, options))
    }
}

/// 変換エンジン
///
/// 複数の `Converter` 実装を保持し、入出力フォーマットに応じて
/// 適切なコンバータを選択・実行する。`CancellationToken` により
/// バッチ処理中のキャンセル機構を提供する。
pub struct ConversionEngine {
    /// 登録されたコンバータ群
    converters: Vec<Box<dyn ConverterBoxed>>,
    /// キャンセルトークン（バッチ処理の中断に使用）
    cancel_token: CancellationToken,
}

impl ConversionEngine {
    /// 新しい ConversionEngine を作成する
    ///
    /// デフォルトで `ImageConverter` と `TextConverter` を登録する。
    pub fn new() -> Self {
        let converters: Vec<Box<dyn ConverterBoxed>> = vec![
            Box::new(image::ImageConverter::new()),
            Box::new(text::TextConverter::new()),
        ];

        Self {
            converters,
            cancel_token: CancellationToken::new(),
        }
    }

    /// 指定されたフォーマットペアに対応するコンバータを検索する
    ///
    /// # Returns
    /// 変換可能なコンバータへの参照。見つからない場合は `None`。
    pub fn find_converter(
        &self,
        from: &FileFormat,
        to: &FileFormat,
    ) -> Option<&dyn ConverterBoxed> {
        self.converters
            .iter()
            .find(|c| c.can_convert(from, to))
            .map(|c| c.as_ref())
    }

    /// 指定されたフォーマットペアの変換が可能かどうかを判定
    pub fn can_convert(&self, from: &FileFormat, to: &FileFormat) -> bool {
        self.find_converter(from, to).is_some()
    }

    /// 現在のキャンセルトークンの子トークンを取得する
    ///
    /// バッチ処理ループ内で `token.is_cancelled()` をチェックし、
    /// キャンセルされている場合は処理を中断する。
    pub fn cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    /// 現在の変換処理をキャンセルする
    ///
    /// キャンセルが発行されると、バッチ処理は現在のファイル処理完了後に停止する。
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// キャンセルトークンをリセットする
    ///
    /// 新しいバッチ処理を開始する前に呼び出す。
    /// 古いトークンを破棄し、新しいトークンを生成する。
    pub fn reset_cancel(&mut self) {
        self.cancel_token = CancellationToken::new();
    }

    /// キャンセルされたかどうかをチェック
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }

    /// 登録されたコンバータの一覧を取得
    pub fn converters(&self) -> &[Box<dyn ConverterBoxed>] {
        &self.converters
    }
}

impl Default for ConversionEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_finds_image_converter() {
        let engine = ConversionEngine::new();
        assert!(engine.can_convert(&FileFormat::Png, &FileFormat::Jpeg));
        assert!(engine.can_convert(&FileFormat::Webp, &FileFormat::Bmp));
        assert!(engine.find_converter(&FileFormat::Png, &FileFormat::Jpeg).is_some());
    }

    #[test]
    fn test_engine_finds_text_converter() {
        let engine = ConversionEngine::new();
        assert!(engine.can_convert(&FileFormat::Json, &FileFormat::Yaml));
        assert!(engine.can_convert(&FileFormat::Csv, &FileFormat::Json));
        assert!(engine.find_converter(&FileFormat::Json, &FileFormat::Yaml).is_some());
    }

    #[test]
    fn test_engine_rejects_unsupported_pair() {
        let engine = ConversionEngine::new();
        // Image to text is not supported
        assert!(!engine.can_convert(&FileFormat::Png, &FileFormat::Json));
        // Same format is not supported
        assert!(!engine.can_convert(&FileFormat::Png, &FileFormat::Png));
        // Unsupported text pair
        assert!(!engine.can_convert(&FileFormat::Csv, &FileFormat::Xml));
    }

    #[test]
    fn test_engine_cancel_token() {
        let mut engine = ConversionEngine::new();

        assert!(!engine.is_cancelled());

        engine.cancel();
        assert!(engine.is_cancelled());

        engine.reset_cancel();
        assert!(!engine.is_cancelled());
    }

    #[test]
    fn test_cancel_token_clone_shares_state() {
        let engine = ConversionEngine::new();
        let token = engine.cancel_token();

        assert!(!token.is_cancelled());
        engine.cancel();
        assert!(token.is_cancelled());
    }
}
