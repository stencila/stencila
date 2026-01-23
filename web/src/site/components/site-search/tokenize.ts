/**
 * Tokenization for search indexing
 *
 * This module implements text tokenization that must produce identical
 * results to the Rust tokenizer in `rust/site/src/search/tokenize.rs`.
 *
 * Tokenization rules:
 * 1. NFD normalize (preserving case for camelCase detection)
 * 2. Fold diacritics (é → e, ü → u)
 * 3. Split on non-alphanumeric boundaries
 * 4. Split code identifiers (camelCase, snake_case, kebab-case) and lowercase
 * 5. Filter tokens < 2 Unicode code points
 * 6. No stemming (keeps cross-language parity simple)
 */

/**
 * Tokenize text into searchable tokens
 *
 * This function must produce identical output to the Rust
 * `tokenize()` function for the same input.
 */
export function tokenize(text: string): string[] {
  const tokens: string[] = []

  // NFD normalize (but don't lowercase yet - we need case for camelCase splitting)
  const normalized = text.normalize('NFD')

  // Process character by character, folding diacritics and splitting on boundaries
  let currentWord = ''

  for (const c of normalized) {
    const codePoint = c.codePointAt(0) ?? 0

    // Skip combining diacritical marks (U+0300-U+036F) - this folds diacritics
    // They appear after NFD normalization (e.g., "e" -> "e" + combining accent)
    if (codePoint >= 0x0300 && codePoint <= 0x036f) {
      continue
    }

    if (isAlphanumeric(c)) {
      currentWord += c
    } else {
      // Non-alphanumeric boundary (whitespace, punctuation, underscore, hyphen)
      if (currentWord.length > 0) {
        // Split camelCase (this also lowercases)
        tokens.push(...splitCamelCase(currentWord))
        currentWord = ''
      }
    }
  }

  // Don't forget the last word
  if (currentWord.length > 0) {
    tokens.push(...splitCamelCase(currentWord))
  }

  // Filter short tokens (< 2 Unicode code points)
  // Note: Use [...t].length not t.length for parity with Rust's chars().count()
  // (JS length counts UTF-16 code units; spread iterates code points)
  return tokens.filter((t) => [...t].length >= 2)
}

/**
 * Check if a character is alphanumeric (letter or digit)
 */
function isAlphanumeric(c: string): boolean {
  // Use a simple regex that matches Unicode letters and digits
  return /^[\p{L}\p{N}]$/u.test(c)
}

/**
 * Split camelCase and PascalCase identifiers into separate tokens
 *
 * Examples:
 * - "camelCase" -> ["camel", "case"]
 * - "PascalCase" -> ["pascal", "case"]
 * - "HTMLParser" -> ["html", "parser"]
 * - "getID" -> ["get", "id"]
 *
 * Note: Uses ASCII-only case detection (A-Z, a-z) for parity with Rust
 * tokenizer which uses is_ascii_uppercase/is_ascii_lowercase. This is
 * intentional - camelCase splitting is primarily for code identifiers
 * which typically use ASCII letters.
 */
function splitCamelCase(word: string): string[] {
  const tokens: string[] = []
  let current = ''
  const chars = [...word]

  for (let i = 0; i < chars.length; i++) {
    const c = chars[i]
    // ASCII-only case detection for Rust parity
    const isUpper = c >= 'A' && c <= 'Z'
    const wasLower = i > 0 && chars[i - 1] >= 'a' && chars[i - 1] <= 'z'
    const nextIsLower =
      i + 1 < chars.length && chars[i + 1] >= 'a' && chars[i + 1] <= 'z'

    // Start new token if:
    // 1. Uppercase after lowercase (camelCase boundary): "camelCase" splits at 'C'
    // 2. Uppercase followed by lowercase in uppercase run (HTMLParser): "HTML" + "Parser"
    const shouldSplit = isUpper && current.length > 0 && (wasLower || nextIsLower)

    if (shouldSplit) {
      tokens.push(current.toLowerCase())
      current = ''
    }

    current += c
  }

  if (current.length > 0) {
    tokens.push(current.toLowerCase())
  }

  return tokens
}

/**
 * Get the 2-character prefix for shard lookup
 *
 * This determines which shard a token belongs to.
 * Uses Unicode code points for parity with Rust.
 */
export function tokenPrefix(token: string): string {
  // Use spread to iterate code points, not UTF-16 code units
  const codePoints = [...token]
  return codePoints.slice(0, Math.min(2, codePoints.length)).join('')
}
