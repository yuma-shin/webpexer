import type { TFunction } from 'i18next'

/**
 * バックエンドから返されたエラー文字列を検出パターンに基づき i18n キーにマッピングし、
 * 翻訳されたメッセージを返す。
 *
 * バックエンドのエラーは Rust の `Display` 実装に基づく日本語文字列として送信される。
 * このユーティリティはエラー文字列のパターンを検出して適切な i18n キーに振り分ける。
 * パターンにマッチしない場合はエラー文字列をそのまま返す。
 */
export function formatError(errorString: string, t: TFunction): string {
  // ディスク容量不足
  const diskSpaceMatch = errorString.match(
    /ディスク容量不足|Insufficient disk space|必要=(\d+),\s*空き=(\d+)/,
  )
  if (diskSpaceMatch) {
    const requiredMatch = errorString.match(/必要=(\d+)/)
    const availableMatch = errorString.match(/空き=(\d+)/)
    if (requiredMatch && availableMatch) {
      return t('errors.diskSpace', {
        required: formatBytes(Number(requiredMatch[1])),
        available: formatBytes(Number(availableMatch[1])),
      })
    }
    return t('errors.diskSpace', { required: '?', available: '?' })
  }

  // 権限エラー
  const permissionMatch = errorString.match(/権限エラー:\s*(.+)|Permission denied:\s*(.+)/)
  if (permissionMatch) {
    const path = permissionMatch[1] || permissionMatch[2] || ''
    return t('errors.permission', { path: path.trim() })
  }

  // ファイルサイズ超過
  const fileSizeMatch = errorString.match(
    /ファイルサイズ超過:\s*(.+?)\s*\((\d+)MB\s*>\s*(\d+)MB\)/,
  )
  if (fileSizeMatch) {
    return t('errors.fileSize', {
      fileName: fileSizeMatch[1],
      size: fileSizeMatch[2],
      limit: fileSizeMatch[3],
    })
  }

  // 非対応変換ペア
  const unsupportedMatch = errorString.match(
    /非対応変換ペア:\s*(.+?)\s*→\s*(.+)|Unsupported conversion:\s*(.+?)\s*(?:to|→)\s*(.+)/,
  )
  if (unsupportedMatch) {
    const from = unsupportedMatch[1] || unsupportedMatch[3] || ''
    const to = unsupportedMatch[2] || unsupportedMatch[4] || ''
    return t('errors.unsupported', { from: from.trim(), to: to.trim() })
  }

  // パースエラー
  const parseMatch = errorString.match(
    /パースエラー:\s*(.+?)\s*行(\d+|None)\s*-\s*(.+)/,
  )
  if (parseMatch) {
    return t('errors.parseError', {
      fileName: parseMatch[1],
      line: parseMatch[2] === 'None' ? '?' : parseMatch[2],
      detail: parseMatch[3],
    })
  }

  // ファイル読み取りエラー
  const fileReadMatch = errorString.match(/ファイル読み取りエラー:\s*(.+?)\s*-\s*(.+)/)
  if (fileReadMatch) {
    return t('errors.fileRead', {
      fileName: fileReadMatch[1],
      detail: fileReadMatch[2],
    })
  }

  // デコードエラー
  const decodeMatch = errorString.match(/デコードエラー:\s*(.+?)\s*-\s*(.+)/)
  if (decodeMatch) {
    return t('errors.decode', {
      fileName: decodeMatch[1],
      detail: decodeMatch[2],
    })
  }

  // エンコードエラー
  const encodeMatch = errorString.match(/エンコードエラー:\s*(.+?)\s*-\s*(.+)/)
  if (encodeMatch) {
    return t('errors.encode', {
      fileName: encodeMatch[1],
      detail: encodeMatch[2],
    })
  }

  // ファイル書き込みエラー
  const writeMatch = errorString.match(/ファイル書き込みエラー:\s*(.+?)\s*-\s*(.+)/)
  if (writeMatch) {
    return t('errors.fileWrite', {
      fileName: writeMatch[1],
      detail: writeMatch[2],
    })
  }

  // フォルダ作成失敗
  const dirMatch = errorString.match(/フォルダ作成失敗:\s*(.+?)\s*-\s*(.+)/)
  if (dirMatch) {
    return t('errors.directoryCreate', {
      path: dirMatch[1],
      detail: dirMatch[2],
    })
  }

  // バリデーションエラー
  const validationMatch = errorString.match(/バリデーションエラー:\s*(.+)/)
  if (validationMatch) {
    return t('errors.validation', { detail: validationMatch[1] })
  }

  // キャンセル
  if (errorString.includes('キャンセルされました') || errorString.includes('Cancelled')) {
    return t('errors.cancelled')
  }

  // パターンにマッチしない場合はそのまま返す
  return errorString
}

/**
 * バイト数を人間が読みやすい形式にフォーマットする
 */
function formatBytes(bytes: number): string {
  if (bytes >= 1_073_741_824) {
    return `${(bytes / 1_073_741_824).toFixed(1)}GB`
  } else if (bytes >= 1_048_576) {
    return `${(bytes / 1_048_576).toFixed(1)}MB`
  } else if (bytes >= 1_024) {
    return `${(bytes / 1_024).toFixed(1)}KB`
  }
  return `${bytes}B`
}
