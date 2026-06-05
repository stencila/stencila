/**
 * Unit tests for shared format-client helpers that translate between protocol
 * code point positions and JavaScript editor indexes.
 */
import { describe, expect, it } from 'vitest'

import {
  applyContentOperation,
  codePointIndexToUtf16Index,
  codePointLength,
  utf16IndexToCodePointIndex,
} from './format'

describe('format client code point helpers', () => {
  it('counts Unicode code points instead of UTF-16 code units', () => {
    expect('a🌍b'.length).toBe(4)
    expect(codePointLength('a🌍b')).toBe(3)
  })

  it('applies content operations at EOF', () => {
    expect(
      applyContentOperation('Hello', {
        type: 'replace',
        from: 0,
        to: 5,
        insert: 'Hi',
      })
    ).toBe('Hi')

    expect(
      applyContentOperation('Hi 🌍', {
        type: 'insert',
        from: 4,
        insert: '!',
      })
    ).toBe('Hi 🌍!')

    expect(
      applyContentOperation('Hi 🌍!', {
        type: 'delete',
        from: 4,
        to: 5,
      })
    ).toBe('Hi 🌍')
  })

  it('converts between protocol and editor indexes', () => {
    const value = 'a🌍b'

    expect(utf16IndexToCodePointIndex(value, 3)).toBe(2)
    expect(codePointIndexToUtf16Index(value, 2)).toBe(3)
    expect(codePointIndexToUtf16Index(value, 3)).toBe(4)
    expect(codePointIndexToUtf16Index(value, 4)).toBeUndefined()
  })
})
