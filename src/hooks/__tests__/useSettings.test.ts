import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import type { SettingsState } from '../useSettings'

// Mock @tauri-apps/plugin-store
const mockGet = vi.fn()
const mockSet = vi.fn()
const mockSave = vi.fn()

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(() =>
    Promise.resolve({
      get: mockGet,
      set: mockSet,
      save: mockSave,
    }),
  ),
}))

// Import after mocks
import { useSettings } from '../useSettings'

const DEFAULT_SETTINGS: SettingsState = {
  quality: 80,
  outputFormat: null,
  outputDir: null,
  fileNaming: 'original',
  conflictResolution: 'overwrite',
  deleteSource: false,
  resize: null,
}

describe('useSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockGet.mockResolvedValue(null)
    mockSet.mockResolvedValue(undefined)
    mockSave.mockResolvedValue(undefined)
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('returns default settings on initial load when store is empty', async () => {
    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    expect(result.current.settings).toEqual(DEFAULT_SETTINGS)
  })

  it('loads stored settings from Tauri Store on mount', async () => {
    const storedSettings: SettingsState = {
      quality: 90,
      outputFormat: 'png',
      outputDir: '/some/path',
      fileNaming: 'original_sequential',
      conflictResolution: 'skip',
      deleteSource: true,
      resize: { width: 800, height: null, maintainAspectRatio: true },
    }
    mockGet.mockResolvedValue(storedSettings)

    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    expect(result.current.settings).toEqual(storedSettings)
  })

  it('merges stored settings with defaults for newly added keys', async () => {
    // Simulate an older stored settings that doesn't have all keys
    const partialStored = {
      quality: 95,
      outputFormat: 'jpeg',
    }
    mockGet.mockResolvedValue(partialStored)

    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    // Should have stored values merged with defaults
    expect(result.current.settings.quality).toBe(95)
    expect(result.current.settings.outputFormat).toBe('jpeg')
    expect(result.current.settings.fileNaming).toBe('original')
    expect(result.current.settings.conflictResolution).toBe('overwrite')
  })

  it('updateSetting updates a single key and persists to store', async () => {
    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    act(() => {
      result.current.updateSetting('quality', 50)
    })

    expect(result.current.settings.quality).toBe(50)

    // Wait for async persistence
    await waitFor(() => {
      expect(mockSet).toHaveBeenCalledWith(
        'conversionSettings',
        expect.objectContaining({ quality: 50 }),
      )
    })
    expect(mockSave).toHaveBeenCalled()
  })

  it('updateSetting preserves other settings when updating one key', async () => {
    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    act(() => {
      result.current.updateSetting('outputFormat', 'webp')
    })

    expect(result.current.settings.outputFormat).toBe('webp')
    expect(result.current.settings.quality).toBe(80)
    expect(result.current.settings.fileNaming).toBe('original')
  })

  it('resetSettings restores all defaults', async () => {
    const storedSettings: SettingsState = {
      quality: 50,
      outputFormat: 'png',
      outputDir: '/custom/dir',
      fileNaming: 'original_datetime',
      conflictResolution: 'rename',
      deleteSource: true,
      resize: { width: 1024, height: 768, maintainAspectRatio: false },
    }
    mockGet.mockResolvedValue(storedSettings)

    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    expect(result.current.settings.quality).toBe(50)

    act(() => {
      result.current.resetSettings()
    })

    expect(result.current.settings).toEqual(DEFAULT_SETTINGS)
  })

  it('handles store load failure gracefully', async () => {
    const { load } = await import('@tauri-apps/plugin-store')
    vi.mocked(load).mockRejectedValueOnce(new Error('Store unavailable'))

    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    // Should still return defaults
    expect(result.current.settings).toEqual(DEFAULT_SETTINGS)
  })

  it('updateSetting for resize works correctly', async () => {
    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    const newResize = { width: 1920, height: null, maintainAspectRatio: true }

    act(() => {
      result.current.updateSetting('resize', newResize)
    })

    expect(result.current.settings.resize).toEqual(newResize)
  })

  it('updateSetting for deleteSource works correctly', async () => {
    const { result } = renderHook(() => useSettings())

    await waitFor(() => {
      expect(result.current.isLoaded).toBe(true)
    })

    act(() => {
      result.current.updateSetting('deleteSource', true)
    })

    expect(result.current.settings.deleteSource).toBe(true)
  })
})
