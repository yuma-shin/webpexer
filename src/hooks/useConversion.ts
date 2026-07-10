import { useCallback, useRef, useState } from 'react'
import type { ConversionParams, ConversionResult, ProgressEvent } from '../types'
import {
  cancelConversion,
  startConversion as invokeStartConversion,
} from '../lib/tauri-commands'

/** 変換処理の状態 */
export type ConversionStatus =
  | 'idle'
  | 'converting'
  | 'completed'
  | 'cancelled'
  | 'error'

export interface UseConversionReturn {
  /** 現在の変換ステータス */
  status: ConversionStatus
  /** 最新の進捗イベント */
  progress: ProgressEvent | null
  /** 変換結果（完了時） */
  result: ConversionResult | null
  /** エラーメッセージ（エラー時） */
  error: string | null
  /** 変換を開始する */
  startConversion: (params: ConversionParams) => Promise<void>
  /** 実行中の変換をキャンセルする */
  cancel: () => Promise<void>
  /** 全状態を初期状態にリセットする */
  reset: () => void
}

/**
 * 変換処理を管理するカスタムフック
 *
 * 変換の開始・キャンセル・進捗監視・結果処理を提供する。
 * Tauri Channel 経由の進捗イベントをリアルタイムで state に反映する。
 */
export function useConversion(): UseConversionReturn {
  const [status, setStatus] = useState<ConversionStatus>('idle')
  const [progress, setProgress] = useState<ProgressEvent | null>(null)
  const [result, setResult] = useState<ConversionResult | null>(null)
  const [error, setError] = useState<string | null>(null)

  // 変換中フラグ（キャンセル後に結果を無視するため）
  const isActiveRef = useRef(false)

  const startConversion = useCallback(async (params: ConversionParams) => {
    // 既に変換中の場合は何もしない
    if (isActiveRef.current) return

    // 状態をリセットして変換開始
    setStatus('converting')
    setProgress(null)
    setResult(null)
    setError(null)
    isActiveRef.current = true

    try {
      const conversionResult = await invokeStartConversion(params, (event) => {
        if (!isActiveRef.current) return

        setProgress(event)

        // キャンセルされた場合のステータス更新
        if (event.status === 'cancelled') {
          setStatus('cancelled')
          isActiveRef.current = false
        }
      })

      // キャンセル済みの場合は結果を無視
      if (!isActiveRef.current) return

      setResult(conversionResult)
      setStatus('completed')
    } catch (err) {
      // キャンセル済みの場合はエラーを無視
      if (!isActiveRef.current) return

      const message = err instanceof Error ? err.message : String(err)
      setError(message)
      setStatus('error')
    } finally {
      isActiveRef.current = false
    }
  }, [])

  const cancel = useCallback(async () => {
    if (!isActiveRef.current) return

    isActiveRef.current = false
    setStatus('cancelled')

    try {
      await cancelConversion()
    } catch {
      // キャンセルコマンドの失敗は無視する
      // (既に完了している場合など)
    }
  }, [])

  const reset = useCallback(() => {
    isActiveRef.current = false
    setStatus('idle')
    setProgress(null)
    setResult(null)
    setError(null)
  }, [])

  return {
    status,
    progress,
    result,
    error,
    startConversion,
    cancel,
    reset,
  }
}
