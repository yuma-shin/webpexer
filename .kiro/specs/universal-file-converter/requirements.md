# Requirements Document

## Introduction

WebPexer（WebP⇔PNG変換デスクトップアプリ）を、汎用ファイル変換アプリとして1から再開発する。フレームワークをElectronからTauriに、UIをMUIからTailwindCSS + HeadlessUIに変更し、画像・テキストファイルなど様々なフォーマットの相互変換に対応する。デザインはStripeのようなシンプルで洗練されたスタイルとする。

## Glossary

- **Converter**: ファイル変換を実行するアプリケーション本体
- **Conversion_Engine**: Tauri バックエンド（Rust）で動作するファイル変換処理モジュール
- **File_Picker**: ユーザーがファイルまたはフォルダを選択するためのUIコンポーネント
- **Format_Selector**: 出力フォーマットを選択するためのUIコンポーネント
- **Conversion_Job**: 1つ以上のファイルを指定フォーマットに変換する処理単位
- **Supported_Format**: アプリケーションが変換可能なファイルフォーマット
- **Progress_Indicator**: 変換進捗を表示するUIコンポーネント
- **Settings_Panel**: アプリケーション設定を管理するUIパネル
- **Source_File**: 変換元のファイル
- **Output_File**: 変換後に生成されるファイル

## Requirements

### Requirement 1: アプリケーション基盤

**User Story:** 開発者として、Tauriベースのデスクトップアプリを構築したい。軽量かつ高速なネイティブアプリケーションを提供するためである。

#### Acceptance Criteria

1. THE Converter SHALL Tauri 2.x フレームワークを使用し、起動後5秒以内にメインウィンドウを表示してデスクトップアプリケーションとして動作する
2. THE Converter SHALL フロントエンドに React 18以上 と TypeScript 5以上 を使用する
3. THE Converter SHALL スタイリングに TailwindCSS と HeadlessUI を使用する
4. THE Converter SHALL バックエンドに Rust を使用し、Tauri のコマンド機構（invoke）を介してフロントエンドからファイル変換処理を呼び出せる
5. THE Converter SHALL Windows 10以降、macOS 12以降、Ubuntu 22.04以降 向けにインストール可能なバイナリとしてビルドできる
6. THE Converter SHALL MIT ライセンスのもとで公開する
7. IF アプリケーション起動時にバックエンド初期化に失敗した場合、THEN THE Converter SHALL エラーを示すメッセージを表示し、ユーザーに再起動を促す

---

### Requirement 2: 画像フォーマット変換

**User Story:** ユーザーとして、様々な画像フォーマット間で変換したい。用途に応じた最適なフォーマットを選択できるようにするためである。

#### Acceptance Criteria

1. THE Conversion_Engine SHALL 以下の入力フォーマットから任意の出力フォーマットへの変換をサポートする: PNG、JPEG、WebP、GIF、BMP、TIFF、AVIF、ICO
2. WHEN ユーザーがソースフォルダを選択し出力フォーマットを指定した場合, THE Conversion_Engine SHALL フォルダ内の対象画像ファイル（出力フォーマットと同一形式を除く）を指定フォーマットに変換し、ソースフォルダ内にフォーマット名のサブフォルダを作成して Output_File を保存する
3. WHEN 画像変換が実行される場合, THE Conversion_Engine SHALL 品質設定（1〜100、デフォルト値: 80）をユーザー指定値に基づいて非可逆フォーマット（JPEG、WebP、AVIF）に適用する
4. WHEN ユーザーがリサイズオプションを指定して画像変換を実行する場合, THE Conversion_Engine SHALL 指定された幅（1〜16383ピクセル）または高さ（1〜16383ピクセル）に基づいて画像をリサイズし、アスペクト比維持が有効な場合は未指定の辺を元画像の比率で算出する
5. IF 入力ファイルの処理中にエラーが発生した場合（ヘッダー読み取り不能、画像デコード失敗、その他のファイル固有エラーを含む）, THEN THE Conversion_Engine SHALL 該当ファイル名を含むエラーメッセージを返し、該当ファイルの変換処理をスキップして残りのファイルの変換を継続する
6. IF 品質設定が可逆フォーマット（PNG、GIF、BMP、TIFF、ICO）に対して指定された場合, THEN THE Conversion_Engine SHALL 品質設定を無視し可逆圧縮で出力する

---

### Requirement 3: テキスト・ドキュメントフォーマット変換

**User Story:** ユーザーとして、テキスト系ファイルのフォーマットを変換したい。異なるツール間でのデータ互換性を確保するためである。

#### Acceptance Criteria

1. THE Conversion_Engine SHALL 以下のテキスト・ドキュメントフォーマット間の構造化データ変換をサポートする: JSON↔YAML、JSON↔TOML、JSON↔XML、CSV→JSON、Markdown→Plain Text、Plain Text→Markdown。サポート対象外の変換ペアを選択した場合、変換不可である旨をユーザーに表示する
2. WHEN JSON を YAML に変換する場合, THE Conversion_Engine SHALL ネスト構造、配列、数値型・文字列型・真偽型・null の区別を保持したまま変換する
3. WHEN CSV を JSON に変換する場合, THE Conversion_Engine SHALL 先頭行をヘッダーとして解釈しオブジェクト配列に変換する。区切り文字はカンマをデフォルトとし、タブおよびセミコロンを選択オプションとして提供する
4. WHEN テキストファイルを変換する場合, THE Conversion_Engine SHALL 入力ファイルのエンコーディングを自動検出し、出力エンコーディングとして UTF-8（デフォルト）、Shift_JIS、EUC-JP から選択するオプションを提供する
5. IF 入力ファイルのパースに失敗した場合, THEN THE Conversion_Engine SHALL パースエラーの行番号と内容をエラーメッセージとして返す
6. THE Conversion_Engine SHALL JSON→YAML→JSON のラウンドトリップ変換において、キー名、値の型（数値・文字列・真偽値・null）、ネスト構造、および配列要素の順序が変換前後で一致する（キーの出現順序は問わない）
7. IF 入力ファイルのサイズが 50MB を超える場合, THEN THE Conversion_Engine SHALL 変換を実行せず、ファイルサイズ上限超過を示すエラーメッセージを返す
8. IF 構造化データ（JSON、YAML、TOML、XML）を CSV に変換する際にネストされたオブジェクトや配列が含まれる場合, THEN THE Conversion_Engine SHALL 変換不可である旨をエラーメッセージとして返し、変換を中止する

---

### Requirement 4: ファイル選択とバッチ処理

**User Story:** ユーザーとして、複数ファイルを一括で変換したい。大量のファイルを効率的に処理するためである。

#### Acceptance Criteria

1. THE File_Picker SHALL ファイルのドラッグ＆ドロップによる選択を受け付ける
2. THE File_Picker SHALL OS ネイティブのファイルダイアログによる複数ファイル選択を受け付ける
3. THE File_Picker SHALL フォルダ選択によるフォルダ内の画像ファイル（現在の出力形式と異なる拡張子を持つファイル）の一括追加を受け付け、サブフォルダも再帰的に探索する
4. WHEN 複数ファイルが選択された場合, THE Conversion_Engine SHALL 全ファイルを順次処理する Conversion_Job を作成し、ユーザーが変換開始を明示的に指示するまで待機する
5. WHILE Conversion_Job が実行中である場合, THE Progress_Indicator SHALL 処理済みファイル数と総ファイル数を表示する
6. WHILE Conversion_Job が実行中である場合, THE Converter SHALL キャンセルボタンを表示し、押下時に現在処理中のファイルの変換完了を待った後、未処理ファイルの変換を中止し、既に変換済みのファイルは出力先に保持する
7. IF Conversion_Job 内の1ファイルの変換が失敗した場合, THEN THE Conversion_Engine SHALL 該当ファイルをスキップし残りのファイルの処理を継続し、処理完了後に失敗したファイルの件数を結果メッセージに含める
8. IF 選択されたフォルダ内に対象ファイルが存在しない場合, THEN THE File_Picker SHALL 対象ファイルが見つからなかった旨のメッセージを表示し、変換処理を開始しない

---

### Requirement 5: 出力設定

**User Story:** ユーザーとして、出力先と出力ファイルの命名規則を設定したい。変換結果を整理して管理するためである。

#### Acceptance Criteria

1. THE Converter SHALL 出力先フォルダをユーザーがフォルダ選択ダイアログにより指定可能である
2. IF 出力先フォルダが指定されていない, THEN THE Converter SHALL Source_File と同じフォルダ内に出力フォーマット名のサブフォルダを作成し、その中に Output_File を生成する
3. THE Converter SHALL 出力ファイル名のパターン設定として、元ファイル名（デフォルト）、元ファイル名+連番（1始まり、ゼロ埋め3桁）、元ファイル名+日時（yyyyMMdd_HHmmss形式）の3種類を提供する
4. IF 同名の Output_File が出力先に既に存在する場合, THEN THE Converter SHALL ユーザーが事前に選択した衝突解決方式（上書き、スキップ、またはリネーム（接尾辞「\_N」Nは2から始まる連番を追加））に従い処理を実行する。デフォルトの衝突解決方式は上書きとする
5. WHERE 元ファイル削除オプションが有効である場合, THE Converter SHALL 変換成功後に Source_File を削除する
6. IF 出力先フォルダが存在しないまたは書き込み権限がない場合, THEN THE Converter SHALL 変換を開始せず、出力先フォルダが利用不可であることを示すエラーメッセージを表示する
7. IF 元ファイル削除オプションが有効かつ Source_File の削除に失敗した場合, THEN THE Converter SHALL 変換済みの Output_File を保持したまま、削除失敗を示すエラーメッセージを表示し、処理を続行する
8. IF サブフォルダの作成に失敗した場合, THEN THE Converter SHALL 変換を開始せず、フォルダ作成失敗を示すエラーメッセージを表示する

---

### Requirement 6: ユーザーインターフェース

**User Story:** ユーザーとして、シンプルで洗練されたUIを使いたい。直感的にファイル変換操作を行えるようにするためである。

#### Acceptance Criteria

1. THE Converter SHALL 白背景、コンポーネント間の余白16px以上、角丸半径4px以上のコンポーネント、elevation 1〜3相当のシャドウを用いたミニマルデザインを採用する
2. THE Converter SHALL ダークモードとライトモードをUIヘッダー内のトグルボタンで切り替え可能とし、起動時はOSのカラースキーム設定に従う
3. THE Converter SHALL メインビューにドラッグ＆ドロップエリア、フォーマット選択、変換ボタンを配置する
4. WHILE ファイルがドラッグ＆ドロップエリア上にホバーされている場合, THE Converter SHALL ドロップエリアの境界線色を変更し、背景色を通常時と異なる色に変化させることでハイライトする
5. WHEN 変換が完了した場合, THE Converter SHALL 成功通知を通知エリアに表示し、出力フォルダを開くボタンを提供する。通知はユーザーが閉じるか、新しい変換が開始されるまで表示する
6. IF 変換中にエラーが発生した場合, THEN THE Converter SHALL エラーの原因を示すメッセージを通知エリアに表示し、ユーザーが手動で閉じるまで表示し続ける
7. THE Converter SHALL 最小ウィンドウサイズ幅400px、高さ300pxを下限とし、それ以上のウィンドウサイズ変更に対してレイアウトが崩れることなく追従する
8. WHEN ダークモードが有効な場合, THE Converter SHALL 背景色を暗色系、テキスト色を明色系に切り替え、すべてのUIコンポーネントが可読性を維持する

---

### Requirement 7: 国際化対応

**User Story:** ユーザーとして、母国語でアプリを使いたい。言語の壁なく操作を行えるようにするためである。

#### Acceptance Criteria

1. THE Converter SHALL 日本語（jp）と英語（en）の 2 言語で UI 表示を切り替え可能である
2. WHEN アプリケーションが起動した場合, THE Converter SHALL OS のシステム言語設定を検出し初期値として自動選択する。ただしユーザーが以前に言語を変更している場合はユーザー設定を永続化し、次回起動時にはユーザー設定を優先する
3. IF OS のシステム言語が対応言語（jp/en）のいずれでもない場合, THEN THE Converter SHALL フォールバック言語として英語（en）を表示する
4. WHEN ユーザーがヘッダー内の言語セレクタで表示言語を変更した場合, THE Converter SHALL アプリケーションの再起動なしに即座に全 UI テキストを選択言語で再描画する
5. THE Converter SHALL UIラベル、ボタンテキスト、プレースホルダー、および変換結果メッセージ（成功・エラー）を翻訳対象とする

---

### Requirement 8: 変換履歴

**User Story:** ユーザーとして、過去の変換履歴を確認したい。以前の設定を再利用して効率的に作業するためである。

#### Acceptance Criteria

1. WHEN 変換処理が完了した場合, THE Converter SHALL 変換履歴（Source フォルダパス、出力フォーマット、クリーンアップ設定、実行日時、結果ステータス（成功または失敗））を記録し、アプリケーション再起動後も保持する
2. THE Converter SHALL 直近50件の変換履歴を実行日時の降順でリスト表示し、記録が50件を超えた場合は最も古い履歴を自動的に削除する
3. WHEN ユーザーが履歴項目を選択した場合, THE Converter SHALL 該当設定（出力フォーマット、クリーンアップ設定）を現在の変換設定に復元する
4. IF 履歴から復元したフォルダパスが存在しない場合, THEN THE Converter SHALL フォルダパスが無効であることを示すメッセージを表示し、フォルダ選択を未選択状態にする
5. WHEN ユーザーが履歴クリアを実行した場合, THE Converter SHALL 確認後、保存されている全変換履歴を削除する

---

### Requirement 9: パフォーマンスとリソース管理

**User Story:** ユーザーとして、大きなファイルや大量のファイルを快適に変換したい。システムリソースを圧迫せずに作業できるようにするためである。

#### Acceptance Criteria

1. WHEN Source_File のサイズが100MB以下である画像ファイルの変換が開始された場合, THE Conversion_Engine SHALL 1ファイルあたり30秒以内に変換を完了する
2. THE Conversion_Engine SHALL 変換処理をバックグラウンドスレッドで実行し、変換中もUIが200ミリ秒以内にユーザー操作に応答する状態を維持する
3. WHILE Conversion_Job が実行中である場合, THE Converter SHALL アプリケーション全体のメモリ使用量を500MB以下に維持する
4. IF Source_File のサイズが100MBを超える場合, THEN THE Converter SHALL 変換開始前にファイルサイズを明示した確認ダイアログを表示し、ユーザーの承認を待つ
5. IF 確認ダイアログでユーザーがキャンセルを選択した場合, THEN THE Converter SHALL 当該ファイルの変換を行わずスキップし、残りのファイルの変換処理を継続する
6. IF 確認ダイアログの表示に失敗した場合, THEN THE Converter SHALL ダイアログなしで変換処理を続行する

---

### Requirement 10: エラーハンドリングとバリデーション

**User Story:** ユーザーとして、不正な操作や予期しないエラーから保護されたい。データ損失を防ぎ安全に操作するためである。

#### Acceptance Criteria

1. WHEN 選択されたフォルダ内に対応フォーマットではないファイルが含まれている場合, THE Converter SHALL 当該ファイルをスキップし、スキップされたファイル名と対応入力フォーマット一覧を含むエラーメッセージを表示する
2. WHEN ユーザーが出力先フォルダとして書き込み権限がないフォルダを選択した場合, THE Converter SHALL 変換を開始せず、権限エラーメッセージを表示しフォルダ再選択を促す
3. IF 変換中にディスク容量不足により書き込みが失敗した場合, THEN THE Converter SHALL 残りのファイルの変換を中断し、正常に完了したファイル数と失敗したファイル名を含むディスク容量不足のエラーメッセージを表示する
4. WHEN 変換処理が開始される前に, THE Converter SHALL 出力先フォルダの空き容量が入力ファイル合計サイズ以上であることを確認し、不足している場合は変換を開始せずに必要容量と現在の空き容量を含むエラーメッセージを表示する
5. IF 元ファイル削除オプションが有効でバッチ変換中に特定ファイルの変換が失敗した場合, THEN THE Converter SHALL 失敗したファイルの元ファイルを削除せずに保持し、正常に変換完了したファイルについてのみ元ファイル削除を実行する
6. IF 変換中にファイル単位でエラーが発生した場合, THEN THE Converter SHALL 当該ファイルの不完全な出力ファイルを削除し、残りのファイルの変換を継続する
