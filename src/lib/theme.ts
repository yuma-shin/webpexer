import { useState, useEffect, useCallback } from 'react'
import { load } from '@tauri-apps/plugin-store'
import type { ThemeMode } from '../types'

const STORE_FILE = 'settings.json'
const STORE_KEY = 'theme'

/**
 * OS のカラースキーム設定を検出する
 */
export function getSystemTheme(): 'light' | 'dark' {
  if (
    typeof window !== 'undefined' &&
    window.matchMedia('(prefers-color-scheme: dark)').matches
  ) {
    return 'dark'
  }
  return 'light'
}

/**
 * テーマモードに応じて document.documentElement の class を切り替える
 */
export function applyTheme(mode: ThemeMode): void {
  const effectiveTheme = mode === 'system' ? getSystemTheme() : mode

  if (effectiveTheme === 'dark') {
    document.documentElement.classList.add('dark')
  } else {
    document.documentElement.classList.remove('dark')
  }
}

/**
 * Tauri Store からテーマ設定を読み込む
 */
export async function getStoredTheme(): Promise<ThemeMode | null> {
  try {
    const store = await load(STORE_FILE)
    const value = await store.get<ThemeMode>(STORE_KEY)
    if (value === 'system' || value === 'light' || value === 'dark') {
      return value
    }
    return null
  } catch {
    return null
  }
}

/**
 * テーマ設定を Tauri Store に保存する
 */
async function saveTheme(mode: ThemeMode): Promise<void> {
  try {
    const store = await load(STORE_FILE)
    await store.set(STORE_KEY, mode)
    await store.save()
  } catch {
    // Store が利用不可の場合は静かに失敗する
  }
}

/**
 * テーマを適用し、Tauri Store に永続化する
 */
export async function setTheme(mode: ThemeMode): Promise<void> {
  applyTheme(mode)
  await saveTheme(mode)
}

/**
 * 初期テーマを読み込み・適用する
 * 保存済みのテーマがなければ "system" をデフォルトとする
 */
export async function initTheme(): Promise<ThemeMode> {
  const stored = await getStoredTheme()
  const mode = stored ?? 'system'
  applyTheme(mode)
  return mode
}

/**
 * テーマ管理 React Hook
 *
 * - `theme`: 現在の ThemeMode ("system" | "light" | "dark")
 * - `setTheme(mode)`: テーマを変更し永続化する
 * - `isDark`: 現在の実効的なダーク状態
 */
export function useTheme() {
  const [theme, setThemeState] = useState<ThemeMode>('system')
  const [isDark, setIsDark] = useState<boolean>(false)

  // 実効テーマ（dark かどうか）を更新するヘルパー
  const updateIsDark = useCallback((mode: ThemeMode) => {
    const effective = mode === 'system' ? getSystemTheme() : mode
    setIsDark(effective === 'dark')
  }, [])

  // 初期化: Store からテーマを読み込み適用
  useEffect(() => {
    let cancelled = false

    initTheme().then((mode) => {
      if (!cancelled) {
        setThemeState(mode)
        updateIsDark(mode)
      }
    })

    return () => {
      cancelled = true
    }
  }, [updateIsDark])

  // OS カラースキーム変更をリッスン（"system" モード時のみ反映）
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

    const handleChange = () => {
      if (theme === 'system') {
        applyTheme('system')
        setIsDark(getSystemTheme() === 'dark')
      }
    }

    mediaQuery.addEventListener('change', handleChange)
    return () => {
      mediaQuery.removeEventListener('change', handleChange)
    }
  }, [theme])

  // テーマ設定変更
  const changeTheme = useCallback(
    async (mode: ThemeMode) => {
      setThemeState(mode)
      updateIsDark(mode)
      await setTheme(mode)
    },
    [updateIsDark],
  )

  return {
    theme,
    setTheme: changeTheme,
    isDark,
  } as const
}
