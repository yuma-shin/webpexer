import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useFileSelection } from '../useFileSelection'

// Mock tauri-commands
vi.mock('../../lib/tauri-commands', () => ({
  selectFolder: vi.fn(),
}))

// Mock @tauri-apps/plugin-dialog
vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
}))

import { selectFolder as mockSelectFolder } from '../../lib/tauri-commands'
import { open as mockOpen } from '@tauri-apps/plugin-dialog'

describe('useFileSelection', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('初期状態で空のファイルリストを返す', () => {
    const { result } = renderHook(() => useFileSelection())
    expect(result.current.files).toEqual([])
  })

  it('addFiles でファイルを追加できる', () => {
    const { result } = renderHook(() => useFileSelection())

    act(() => {
      result.current.addFiles(['/path/to/file1.png', '/path/to/file2.jpg'])
    })

    expect(result.current.files).toEqual(['/path/to/file1.png', '/path/to/file2.jpg'])
  })

  it('addFiles で重複パスを排除する', () => {
    const { result } = renderHook(() => useFileSelection())

    act(() => {
      result.current.addFiles(['/path/to/file1.png', '/path/to/file2.jpg'])
    })

    act(() => {
      result.current.addFiles(['/path/to/file2.jpg', '/path/to/file3.bmp'])
    })

    expect(result.current.files).toEqual([
      '/path/to/file1.png',
      '/path/to/file2.jpg',
      '/path/to/file3.bmp',
    ])
  })

  it('removeFile で指定インデックスのファイルを削除できる', () => {
    const { result } = renderHook(() => useFileSelection())

    act(() => {
      result.current.addFiles(['/a.png', '/b.png', '/c.png'])
    })

    act(() => {
      result.current.removeFile(1)
    })

    expect(result.current.files).toEqual(['/a.png', '/c.png'])
  })

  it('removeFile で無効なインデックスは無視される', () => {
    const { result } = renderHook(() => useFileSelection())

    act(() => {
      result.current.addFiles(['/a.png', '/b.png'])
    })

    act(() => {
      result.current.removeFile(-1)
    })

    act(() => {
      result.current.removeFile(5)
    })

    expect(result.current.files).toEqual(['/a.png', '/b.png'])
  })

  it('clearFiles で全ファイルをクリアする', () => {
    const { result } = renderHook(() => useFileSelection())

    act(() => {
      result.current.addFiles(['/a.png', '/b.png', '/c.png'])
    })

    act(() => {
      result.current.clearFiles()
    })

    expect(result.current.files).toEqual([])
  })

  it('selectFilesDialog でファイルダイアログからファイルを追加する', async () => {
    vi.mocked(mockOpen).mockResolvedValue(['/selected/file1.png', '/selected/file2.jpg'])

    const { result } = renderHook(() => useFileSelection())

    await act(async () => {
      await result.current.selectFilesDialog()
    })

    expect(mockOpen).toHaveBeenCalledWith({
      multiple: true,
      directory: false,
    })
    expect(result.current.files).toEqual(['/selected/file1.png', '/selected/file2.jpg'])
  })

  it('selectFilesDialog でキャンセル時は何もしない', async () => {
    vi.mocked(mockOpen).mockResolvedValue(null)

    const { result } = renderHook(() => useFileSelection())

    await act(async () => {
      await result.current.selectFilesDialog()
    })

    expect(result.current.files).toEqual([])
  })

  it('selectFilesDialog で単一ファイル選択も処理できる', async () => {
    vi.mocked(mockOpen).mockResolvedValue('/single/file.png')

    const { result } = renderHook(() => useFileSelection())

    await act(async () => {
      await result.current.selectFilesDialog()
    })

    expect(result.current.files).toEqual(['/single/file.png'])
  })

  it('selectFolderDialog でフォルダ選択ダイアログからフォルダを追加する', async () => {
    vi.mocked(mockSelectFolder).mockResolvedValue('/selected/folder')

    const { result } = renderHook(() => useFileSelection())

    await act(async () => {
      await result.current.selectFolderDialog()
    })

    expect(mockSelectFolder).toHaveBeenCalled()
    expect(result.current.files).toEqual(['/selected/folder'])
  })

  it('selectFolderDialog でキャンセル時は何もしない', async () => {
    vi.mocked(mockSelectFolder).mockResolvedValue(null)

    const { result } = renderHook(() => useFileSelection())

    await act(async () => {
      await result.current.selectFolderDialog()
    })

    expect(result.current.files).toEqual([])
  })

  it('selectFolderDialog でエラー時は何もしない', async () => {
    vi.mocked(mockSelectFolder).mockRejectedValue(new Error('dialog error'))

    const { result } = renderHook(() => useFileSelection())

    await act(async () => {
      await result.current.selectFolderDialog()
    })

    expect(result.current.files).toEqual([])
  })
})
