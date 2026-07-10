import { useState, useCallback } from 'react'
import { open } from '@tauri-apps/plugin-dialog'
import { selectFolder } from '../lib/tauri-commands'

/**
 * ファイル選択フック
 *
 * ドラッグ＆ドロップ、ファイルダイアログ、フォルダダイアログによる
 * ファイルパスリスト管理を提供する。
 *
 * フォルダ選択時のサブフォルダ再帰探索はバックエンド側で処理される
 * （start_conversion に渡された時点で再帰探索が行われる）。
 */
export function useFileSelection() {
  const [files, setFiles] = useState<string[]>([])

  /**
   * ファイルパスをリストに追加する（重複排除）
   */
  const addFiles = useCallback((paths: string[]) => {
    setFiles((prev) => {
      const existing = new Set(prev)
      const newPaths = paths.filter((p) => !existing.has(p))
      if (newPaths.length === 0) return prev
      return [...prev, ...newPaths]
    })
  }, [])

  /**
   * 指定インデックスのファイルをリストから削除する
   */
  const removeFile = useCallback((index: number) => {
    setFiles((prev) => {
      if (index < 0 || index >= prev.length) return prev
      return [...prev.slice(0, index), ...prev.slice(index + 1)]
    })
  }, [])

  /**
   * 全ファイルをクリアする
   */
  const clearFiles = useCallback(() => {
    setFiles([])
  }, [])

  /**
   * ネイティブファイル選択ダイアログを開く（複数選択対応）
   */
  const selectFilesDialog = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        directory: false,
      })
      if (selected) {
        // open() returns string | string[] | null
        const paths = Array.isArray(selected) ? selected : [selected]
        if (paths.length > 0) {
          addFiles(paths)
        }
      }
    } catch {
      // Dialog cancelled or error - do nothing
    }
  }, [addFiles])

  /**
   * ネイティブフォルダ選択ダイアログを開く
   *
   * バックエンド側の select_folder コマンドを使用する。
   * フォルダ内のサブフォルダ再帰探索は変換実行時にバックエンド側で処理される。
   */
  const selectFolderDialog = useCallback(async () => {
    try {
      const folder = await selectFolder()
      if (folder) {
        addFiles([folder])
      }
    } catch {
      // Dialog cancelled or error - do nothing
    }
  }, [addFiles])

  return {
    files,
    addFiles,
    removeFile,
    clearFiles,
    selectFilesDialog,
    selectFolderDialog,
  }
}
