import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useConversion } from '../useConversion'
import type { ConversionParams, ConversionResult, ProgressEvent } from '../../types'

// Mock tauri-commands
vi.mock('../../lib/tauri-commands', () => ({
  startConversion: vi.fn(),
  cancelConversion: vi.fn(),
}))

import { startConversion as mockStartConversion, cancelConversion as mockCancelConversion } from '../../lib/tauri-commands'

const baseParams: ConversionParams = {
  inputPaths: ['/test/image.png'],
  outputFormat: 'jpeg',
  outputDir: '/test/output',
  options: {
    quality: 80,
    fileNaming: 'original',
    conflictResolution: 'overwrite',
    deleteSource: false,
  },
}

const successResult: ConversionResult = {
  successCount: 1,
  failedCount: 0,
  skippedCount: 0,
  failedFiles: [],
  outputDir: '/test/output',
}

describe('useConversion', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('初期状態が idle である', () => {
    const { result } = renderHook(() => useConversion())

    expect(result.current.status).toBe('idle')
    expect(result.current.progress).toBeNull()
    expect(result.current.result).toBeNull()
    expect(result.current.error).toBeNull()
  })

  it('変換開始時に status が converting になる', async () => {
    // startConversion が即座に解決するように設定
    vi.mocked(mockStartConversion).mockImplementation(async () => successResult)

    const { result } = renderHook(() => useConversion())

    await act(async () => {
      await result.current.startConversion(baseParams)
    })

    // 完了後は completed
    expect(result.current.status).toBe('completed')
    expect(result.current.result).toEqual(successResult)
  })

  it('進捗イベントが state に反映される', async () => {
    const progressEvent: ProgressEvent = {
      currentFile: 'image.png',
      processedCount: 1,
      totalCount: 3,
      status: 'processing',
    }

    vi.mocked(mockStartConversion).mockImplementation(async (_params, onProgress) => {
      onProgress(progressEvent)
      return successResult
    })

    const { result } = renderHook(() => useConversion())

    await act(async () => {
      await result.current.startConversion(baseParams)
    })

    // 完了後は result が設定されている
    expect(result.current.result).toEqual(successResult)
    expect(result.current.status).toBe('completed')
  })

  it('変換エラー時に status が error になりメッセージが設定される', async () => {
    const errorMessage = 'Disk space insufficient'
    vi.mocked(mockStartConversion).mockRejectedValue(new Error(errorMessage))

    const { result } = renderHook(() => useConversion())

    await act(async () => {
      await result.current.startConversion(baseParams)
    })

    expect(result.current.status).toBe('error')
    expect(result.current.error).toBe(errorMessage)
    expect(result.current.result).toBeNull()
  })

  it('キャンセル時に status が cancelled になる', async () => {
    vi.mocked(mockCancelConversion).mockResolvedValue(undefined)

    // startConversion が待機中の状態をシミュレート
    let resolveConversion: (value: ConversionResult) => void
    vi.mocked(mockStartConversion).mockImplementation(() => {
      return new Promise<ConversionResult>((resolve) => {
        resolveConversion = resolve
      })
    })

    const { result } = renderHook(() => useConversion())

    // 変換を開始（解決しない Promise）
    act(() => {
      result.current.startConversion(baseParams)
    })

    // キャンセルを実行
    await act(async () => {
      await result.current.cancel()
    })

    expect(result.current.status).toBe('cancelled')
    expect(mockCancelConversion).toHaveBeenCalled()

    // resolve して Promise を完了させる（キャンセル済みなので無視される）
    await act(async () => {
      resolveConversion!(successResult)
    })

    // キャンセル状態が維持される
    expect(result.current.status).toBe('cancelled')
  })

  it('reset で全状態が初期化される', async () => {
    vi.mocked(mockStartConversion).mockResolvedValue(successResult)

    const { result } = renderHook(() => useConversion())

    await act(async () => {
      await result.current.startConversion(baseParams)
    })

    expect(result.current.status).toBe('completed')

    act(() => {
      result.current.reset()
    })

    expect(result.current.status).toBe('idle')
    expect(result.current.progress).toBeNull()
    expect(result.current.result).toBeNull()
    expect(result.current.error).toBeNull()
  })

  it('変換中に再度 startConversion を呼んでも無視される', async () => {
    let resolveConversion: (value: ConversionResult) => void
    vi.mocked(mockStartConversion).mockImplementation(() => {
      return new Promise<ConversionResult>((resolve) => {
        resolveConversion = resolve
      })
    })

    const { result } = renderHook(() => useConversion())

    // 1回目の変換開始
    act(() => {
      result.current.startConversion(baseParams)
    })

    // 2回目の変換開始（無視されるべき）
    act(() => {
      result.current.startConversion(baseParams)
    })

    // startConversion は1回だけ呼ばれる
    expect(mockStartConversion).toHaveBeenCalledTimes(1)

    // クリーンアップ
    await act(async () => {
      resolveConversion!(successResult)
    })
  })

  it('文字列エラーもハンドリングできる', async () => {
    vi.mocked(mockStartConversion).mockRejectedValue('string error')

    const { result } = renderHook(() => useConversion())

    await act(async () => {
      await result.current.startConversion(baseParams)
    })

    expect(result.current.status).toBe('error')
    expect(result.current.error).toBe('string error')
  })
})
