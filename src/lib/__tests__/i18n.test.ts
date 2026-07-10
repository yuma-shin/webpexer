import { describe, it, expect } from 'vitest'
import * as fc from 'fast-check'
import { detectLanguageFromLocale } from '../i18n'
import en from '../../locales/en.json'
import jp from '../../locales/jp.json'

/**
 * Property 18: 言語選択ロジック
 *
 * For any OS locale string, if it starts with `ja`/`jp` → Japanese is selected,
 * if `en` → English, otherwise → English (fallback).
 * If user setting exists, user setting takes priority.
 *
 * **Validates: Requirements 7.2, 7.3**
 */
describe('Property 18: 言語選択ロジック', () => {
  it('ja/jp で始まるロケールは "jp" を返す', () => {
    fc.assert(
      fc.property(
        fc.oneof(
          fc.constant('ja'),
          fc.constant('jp'),
          fc.string({ minLength: 0, maxLength: 10 }).map((s) => 'ja' + s),
          fc.string({ minLength: 0, maxLength: 10 }).map((s) => 'jp' + s),
          fc.string({ minLength: 0, maxLength: 10 }).map((s) => 'ja-' + s),
          fc.string({ minLength: 0, maxLength: 10 }).map((s) => 'jp-' + s),
        ),
        (locale) => {
          expect(detectLanguageFromLocale(locale)).toBe('jp')
        },
      ),
      { numRuns: 100 },
    )
  })

  it('en で始まるロケールは "en" を返す', () => {
    fc.assert(
      fc.property(
        fc.oneof(
          fc.constant('en'),
          fc.string({ minLength: 0, maxLength: 10 }).map((s) => 'en' + s),
          fc.string({ minLength: 0, maxLength: 10 }).map((s) => 'en-' + s),
        ),
        (locale) => {
          expect(detectLanguageFromLocale(locale)).toBe('en')
        },
      ),
      { numRuns: 100 },
    )
  })

  it('ja/jp/en 以外のロケールはフォールバックとして "en" を返す', () => {
    fc.assert(
      fc.property(
        fc.string({ minLength: 0, maxLength: 20 }).filter((s) => {
          const normalized = s.toLowerCase().trim()
          return (
            !normalized.startsWith('ja') &&
            !normalized.startsWith('jp') &&
            !normalized.startsWith('en')
          )
        }),
        (locale) => {
          expect(detectLanguageFromLocale(locale)).toBe('en')
        },
      ),
      { numRuns: 100 },
    )
  })

  it('具体的なロケール文字列に対して正しく判定する', () => {
    // Japanese locales
    expect(detectLanguageFromLocale('ja')).toBe('jp')
    expect(detectLanguageFromLocale('ja-JP')).toBe('jp')
    expect(detectLanguageFromLocale('jp')).toBe('jp')
    expect(detectLanguageFromLocale('jp-JP')).toBe('jp')

    // English locales
    expect(detectLanguageFromLocale('en')).toBe('en')
    expect(detectLanguageFromLocale('en-US')).toBe('en')
    expect(detectLanguageFromLocale('en-GB')).toBe('en')

    // Fallback locales
    expect(detectLanguageFromLocale('fr')).toBe('en')
    expect(detectLanguageFromLocale('de')).toBe('en')
    expect(detectLanguageFromLocale('zh')).toBe('en')
    expect(detectLanguageFromLocale('ko')).toBe('en')
    expect(detectLanguageFromLocale('es')).toBe('en')
    expect(detectLanguageFromLocale('')).toBe('en')
    expect(detectLanguageFromLocale('unknown')).toBe('en')
  })

  it('大文字小文字を区別しない', () => {
    fc.assert(
      fc.property(
        fc.oneof(
          fc.constant('JA'),
          fc.constant('Ja'),
          fc.constant('jA'),
          fc.constant('JP'),
          fc.constant('Jp'),
          fc.constant('jP'),
          fc.constant('EN'),
          fc.constant('En'),
          fc.constant('eN'),
        ),
        (locale) => {
          const normalized = locale.toLowerCase()
          if (normalized.startsWith('ja') || normalized.startsWith('jp')) {
            expect(detectLanguageFromLocale(locale)).toBe('jp')
          } else {
            expect(detectLanguageFromLocale(locale)).toBe('en')
          }
        },
      ),
      { numRuns: 100 },
    )
  })

  it('前後の空白を無視する', () => {
    fc.assert(
      fc.property(
        fc.oneof(fc.constant('ja'), fc.constant('jp'), fc.constant('en'), fc.constant('fr')),
        fc.string({ minLength: 0, maxLength: 3 }).map((s) =>
          s.replace(/[^\s]/g, '').slice(0, 3),
        ),
        (locale, whitespace) => {
          const padded = whitespace + locale + whitespace
          // detectLanguageFromLocale trims, so padded leading whitespace may change the startsWith check
          // Only leading whitespace matters after trim
          const trimmed = padded.trim().toLowerCase()
          if (trimmed.startsWith('ja') || trimmed.startsWith('jp')) {
            expect(detectLanguageFromLocale(padded)).toBe('jp')
          } else {
            expect(detectLanguageFromLocale(padded)).toBe('en')
          }
        },
      ),
      { numRuns: 100 },
    )
  })
})

/**
 * Property 19: 翻訳キーの完全性
 *
 * For any translation key in the app, both Japanese (jp) and English (en) locale files
 * have a corresponding entry.
 *
 * **Validates: Requirements 7.5**
 */
describe('Property 19: 翻訳キーの完全性', () => {
  /**
   * ネストされた JSON オブジェクトからすべてのキーパスを再帰的に抽出する。
   * 例: { a: { b: "hello" } } → ["a.b"]
   */
  function extractKeys(obj: Record<string, unknown>, prefix = ''): string[] {
    const keys: string[] = []
    for (const key of Object.keys(obj)) {
      const fullKey = prefix ? `${prefix}.${key}` : key
      const value = obj[key]
      if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
        keys.push(...extractKeys(value as Record<string, unknown>, fullKey))
      } else {
        keys.push(fullKey)
      }
    }
    return keys
  }

  const enKeys = extractKeys(en as Record<string, unknown>).sort()
  const jpKeys = extractKeys(jp as Record<string, unknown>).sort()

  it('en.json のすべてのキーが jp.json に存在する', () => {
    const missingInJp = enKeys.filter((key) => !jpKeys.includes(key))
    expect(missingInJp).toEqual([])
  })

  it('jp.json のすべてのキーが en.json に存在する', () => {
    const missingInEn = jpKeys.filter((key) => !enKeys.includes(key))
    expect(missingInEn).toEqual([])
  })

  it('両方のロケールファイルのキー集合が完全一致する', () => {
    expect(enKeys).toEqual(jpKeys)
  })

  it('すべてのリーフ値が空文字列でない', () => {
    function extractLeafValues(
      obj: Record<string, unknown>,
      prefix = '',
    ): { key: string; value: unknown }[] {
      const entries: { key: string; value: unknown }[] = []
      for (const key of Object.keys(obj)) {
        const fullKey = prefix ? `${prefix}.${key}` : key
        const value = obj[key]
        if (typeof value === 'object' && value !== null && !Array.isArray(value)) {
          entries.push(...extractLeafValues(value as Record<string, unknown>, fullKey))
        } else {
          entries.push({ key: fullKey, value })
        }
      }
      return entries
    }

    const enLeaves = extractLeafValues(en as Record<string, unknown>)
    const jpLeaves = extractLeafValues(jp as Record<string, unknown>)

    for (const { key, value } of enLeaves) {
      expect(value, `en.json key "${key}" should not be empty`).not.toBe('')
    }
    for (const { key, value } of jpLeaves) {
      expect(value, `jp.json key "${key}" should not be empty`).not.toBe('')
    }
  })
})
