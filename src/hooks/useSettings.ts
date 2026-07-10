import { useState, useEffect, useCallback, useRef } from 'react'
import { load } from '@tauri-apps/plugin-store'
import type {
  FileFormat,
  FileNamingPattern,
  ConflictResolution,
  ResizeOptions,
} from '../types'

// --- Settings shape managed by this hook ---

export interface SettingsState {
  quality: number
  outputFormat: FileFormat | null
  outputDir: string | null
  fileNaming: FileNamingPattern
  conflictResolution: ConflictResolution
  deleteSource: boolean
  resize: ResizeOptions | null
}

// --- Defaults ---

const DEFAULT_SETTINGS: SettingsState = {
  quality: 80,
  outputFormat: null,
  outputDir: null,
  fileNaming: 'original',
  conflictResolution: 'overwrite',
  deleteSource: false,
  resize: null,
}

// --- Store constants ---

const STORE_FILE = 'settings.json'
const STORE_KEY = 'conversionSettings'

// --- Hook ---

export interface UseSettingsReturn {
  settings: SettingsState
  updateSetting: <K extends keyof SettingsState>(
    key: K,
    value: SettingsState[K],
  ) => void
  resetSettings: () => void
  isLoaded: boolean
}

/**
 * 設定管理フック
 *
 * - マウント時に Tauri Store Plugin の `settings.json` から設定を読み込む
 * - 設定変更時に自動的にストアに永続化する
 * - `updateSetting(key, value)` で個別設定を更新
 * - `resetSettings()` でデフォルト設定に復元
 */
export function useSettings(): UseSettingsReturn {
  const [settings, setSettings] = useState<SettingsState>(DEFAULT_SETTINGS)
  const [isLoaded, setIsLoaded] = useState(false)

  // Skip persisting on the initial load to avoid a redundant write
  const isInitialLoad = useRef(true)

  // Load settings from store on mount
  useEffect(() => {
    let cancelled = false

    async function loadSettings() {
      try {
        const store = await load(STORE_FILE)
        const stored = await store.get<SettingsState>(STORE_KEY)
        if (!cancelled && stored) {
          // Merge with defaults to handle any newly added keys
          setSettings({ ...DEFAULT_SETTINGS, ...stored })
        }
      } catch {
        // Store unavailable — use defaults silently
      } finally {
        if (!cancelled) {
          setIsLoaded(true)
        }
      }
    }

    loadSettings()

    return () => {
      cancelled = true
    }
  }, [])

  // Persist settings to store whenever they change (skip first load)
  useEffect(() => {
    if (isInitialLoad.current) {
      isInitialLoad.current = false
      return
    }

    async function persistSettings() {
      try {
        const store = await load(STORE_FILE)
        await store.set(STORE_KEY, settings)
        await store.save()
      } catch {
        // Store unavailable — fail silently
      }
    }

    persistSettings()
  }, [settings])

  // Update a single setting key
  const updateSetting = useCallback(
    <K extends keyof SettingsState>(key: K, value: SettingsState[K]) => {
      setSettings((prev) => ({ ...prev, [key]: value }))
    },
    [],
  )

  // Reset all settings to defaults
  const resetSettings = useCallback(() => {
    setSettings(DEFAULT_SETTINGS)
  }, [])

  return {
    settings,
    updateSetting,
    resetSettings,
    isLoaded,
  }
}
