import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import { load, Store } from '@tauri-apps/plugin-store'
import en from '../locales/en.json'
import jp from '../locales/jp.json'

const STORE_FILE = 'settings.json'
const LANGUAGE_KEY = 'language'
const SUPPORTED_LANGUAGES = ['en', 'jp'] as const

export type SupportedLanguage = (typeof SUPPORTED_LANGUAGES)[number]

let store: Store | null = null

/**
 * OS のロケール文字列から使用言語を判定する。
 * - `ja` または `jp` プレフィックス → "jp"
 * - `en` プレフィックス → "en"
 * - それ以外 → "en"（フォールバック）
 */
export function detectLanguageFromLocale(locale: string): SupportedLanguage {
  const normalized = locale.toLowerCase().trim()
  if (normalized.startsWith('ja') || normalized.startsWith('jp')) {
    return 'jp'
  }
  return 'en'
}

/**
 * Tauri Store から保存済みの言語設定を取得する。
 * ストア未初期化または値なしの場合は null を返す。
 */
export async function getStoredLanguage(): Promise<SupportedLanguage | null> {
  try {
    store = await load(STORE_FILE)
    const lang = await store.get<string>(LANGUAGE_KEY)
    if (lang && SUPPORTED_LANGUAGES.includes(lang as SupportedLanguage)) {
      return lang as SupportedLanguage
    }
    return null
  } catch {
    return null
  }
}

/**
 * 言語を変更し、Tauri Store に永続化する。
 */
export async function changeLanguage(lang: SupportedLanguage): Promise<void> {
  await i18n.changeLanguage(lang)
  try {
    if (!store) {
      store = await load(STORE_FILE)
    }
    await store.set(LANGUAGE_KEY, lang)
    await store.save()
  } catch {
    // Store 書き込み失敗時もUI言語は変更済みなので継続
  }
}

/**
 * i18next を初期化する。
 * 1. Tauri Store に保存済み言語があればそれを使用
 * 2. なければ navigator.language から OS ロケールを検出
 * 3. 検出不可またはサポート外であれば英語にフォールバック
 */
export async function initI18n(): Promise<void> {
  const storedLang = await getStoredLanguage()
  const detectedLang = detectLanguageFromLocale(navigator.language || 'en')
  const initialLang = storedLang ?? detectedLang

  await i18n.use(initReactI18next).init({
    resources: {
      en: { translation: en },
      jp: { translation: jp },
    },
    lng: initialLang,
    fallbackLng: 'en',
    interpolation: {
      escapeValue: false, // React handles XSS
    },
  })
}

export default i18n
