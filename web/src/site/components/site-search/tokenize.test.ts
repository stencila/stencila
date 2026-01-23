/**
 * Tests for tokenization
 *
 * These test vectors MUST match the Rust tests in
 * `rust/site/src/search/tokenize.rs` exactly to ensure
 * cross-language parity.
 */

import { describe, expect, it } from 'vitest'

import { tokenize, tokenPrefix } from './tokenize'

describe('tokenize', () => {
  it('basic tokenization', () => {
    expect(tokenize('hello world')).toEqual(['hello', 'world'])
    expect(tokenize('Hello World')).toEqual(['hello', 'world'])
    expect(tokenize('HELLO WORLD')).toEqual(['hello', 'world'])
  })

  it('diacritic folding', () => {
    expect(tokenize('café')).toEqual(['cafe'])
    expect(tokenize('naïve')).toEqual(['naive'])
    expect(tokenize('résumé')).toEqual(['resume'])
    expect(tokenize('Zürich')).toEqual(['zurich'])
    expect(tokenize('São Paulo')).toEqual(['sao', 'paulo'])
  })

  it('camelCase splitting', () => {
    expect(tokenize('camelCase')).toEqual(['camel', 'case'])
    expect(tokenize('PascalCase')).toEqual(['pascal', 'case'])
    expect(tokenize('HTMLParser')).toEqual(['html', 'parser'])
    expect(tokenize('getID')).toEqual(['get', 'id'])
    expect(tokenize('parseXMLDocument')).toEqual(['parse', 'xml', 'document'])
  })

  it('snake_case splitting', () => {
    expect(tokenize('snake_case')).toEqual(['snake', 'case'])
    expect(tokenize('SCREAMING_SNAKE')).toEqual(['screaming', 'snake'])
    expect(tokenize('mixed_camelCase')).toEqual(['mixed', 'camel', 'case'])
  })

  it('kebab-case splitting', () => {
    expect(tokenize('kebab-case')).toEqual(['kebab', 'case'])
    expect(tokenize('my-component-name')).toEqual(['my', 'component', 'name'])
  })

  it('file paths', () => {
    expect(tokenize('src/components/Button.tsx')).toEqual([
      'src',
      'components',
      'button',
      'tsx',
    ])
    expect(tokenize('my-project/README.md')).toEqual([
      'my',
      'project',
      'readme',
      'md',
    ])
  })

  it('short token filtering', () => {
    expect(tokenize('a b c')).toEqual([])
    expect(tokenize('I am a test')).toEqual(['am', 'test'])
    expect(tokenize('x = 42')).toEqual(['42'])
    // Single non-ASCII characters should be filtered (1 Unicode char < 2)
    expect(tokenize('你')).toEqual([])
    expect(tokenize('你 好')).toEqual([]) // Both are single chars
    // But two-char CJK words should pass
    expect(tokenize('你好')).toEqual(['你好'])
  })

  it('punctuation handling', () => {
    expect(tokenize('hello, world!')).toEqual(['hello', 'world'])
    expect(tokenize("what's up?")).toEqual(['what', 'up'])
    expect(tokenize('test@example.com')).toEqual(['test', 'example', 'com'])
  })

  it('numbers', () => {
    expect(tokenize('test123')).toEqual(['test123'])
    expect(tokenize('123test')).toEqual(['123test'])
    expect(tokenize('test 123 more')).toEqual(['test', '123', 'more'])
  })

  it('empty and whitespace', () => {
    expect(tokenize('')).toEqual([])
    expect(tokenize('   ')).toEqual([])
    expect(tokenize('\n\t')).toEqual([])
  })
})

describe('tokenPrefix', () => {
  it('returns 2-character prefix', () => {
    expect(tokenPrefix('hello')).toBe('he')
    expect(tokenPrefix('a')).toBe('a')
    expect(tokenPrefix('ab')).toBe('ab')
    expect(tokenPrefix('abc')).toBe('ab')
  })
})

describe('astral unicode', () => {
  it('counts astral characters as single code points', () => {
    // Astral characters (outside BMP, U+10000+) should be counted as single code points
    // 𝒜 = U+1D49C (Mathematical Script Capital A) - 1 code point, 2 UTF-16 code units
    expect(tokenize('𝒜')).toEqual([]) // 1 char, filtered
    expect(tokenize('𝒜𝒷')).toEqual(['𝒜𝒷']) // 2 chars, kept
  })

  it('tokenPrefix uses code points not UTF-16 units', () => {
    expect(tokenPrefix('𝒜𝒷𝒸')).toBe('𝒜𝒷') // First 2 code points
    expect(tokenPrefix('𝒜bc')).toBe('𝒜b') // Mixed astral and ASCII
  })
})
