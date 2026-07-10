import { useState, useEffect, useCallback } from 'react'
import type { HistoryEntry, FileFormat } from '../types'
import {
  getConversionHistory,
  clearConversionHistory,
} from '../lib/tauri-commands'

export interface UseHistoryReturn {
  /** 履歴エントリ一覧（降順） */
  entries: HistoryEntry[]
  /** 履歴を全件クリアする */
  clearHistory: () => Promise<void>
  /** 履歴エントリから設定を復元し、結果を返す */
  restoreSettings: (entry: HistoryEntry) => RestoredSettings
  /** 読み込み完了フラグ */
  isLoaded: boolean
  /** 読み込み中のエラー */
  error: string | null
}

/** 履歴から復元された設定 */
export interface RestoredSettings {
  outputFormat: FileFormat
  deleteSource: boolean
}

/**
 * 変換履歴を管理するカスタムフック
 *
 * - マウント時にバックエンドから履歴を読み込む
 * - 最大50件を降順で保持
 * - 履歴選択時の設定復元ロジックを提供
 */
export function useHistory(): UseHistoryReturn {
  const [entries, setEntries] = useState<HistoryEntry[]>([])
  const [isLoaded, setIsLoaded] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // マウント時に履歴を読み込む
  useEffect(() => {
    let cancelled = false

    async function loadHistory() {
      try {
        const history = await getConversionHistory()
        if (!cancelled) {
          setEntries(history)
        }
      } catch (err) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err)
          setError(message)
        }
      } finally {
        if (!cancelled) {
          setIsLoaded(true)
        }
      }
    }

    loadHistory()

    return () => {
      cancelled = true
    }
  }, [])

  // 履歴を全件クリア
  const clearHistory = useCallback(async () => {
    try {
      await clearConversionHistory()
      setEntries([])
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err)
      setError(message)
    }
  }, [])

  // 履歴エントリから設定を復元
  const restoreSettings = useCallback((entry: HistoryEntry): RestoredSettings => {
    return {
      outputFormat: entry.outputFormat,
      deleteSource: entry.cleanupEnabled,
    }
  }, [])

  return {
    entries,
    clearHistory,
    restoreSettings,
    isLoaded,
    error,
  }
}
