/**
 * Search engine for querying the search index
 *
 * Handles tokenization, shard loading, scoring, and result ranking.
 */

import { SearchIndexLoader } from './loader'
import { tokenize, tokenPrefix } from './tokenize'
import type {
  SearchEntry,
  SearchOptions,
  SearchResult,
  TextHighlight,
} from './types'

/**
 * Default search options
 */
const DEFAULT_OPTIONS: Required<SearchOptions> = {
  limit: 20,
  offset: 0,
  nodeTypes: [],
  routes: [],
}

/**
 * Search engine for querying the index
 */
export class SearchEngine {
  private loader: SearchIndexLoader

  constructor(basePath: string = '/_search') {
    this.loader = new SearchIndexLoader(basePath)
  }

  /**
   * Initialize the search engine by loading the manifest
   */
  async initialize(): Promise<void> {
    await this.loader.loadManifest()
  }

  /**
   * Check if the engine is initialized
   */
  isReady(): boolean {
    return this.loader.isLoaded()
  }

  /**
   * Execute a search query
   */
  async search(
    query: string,
    options: SearchOptions = {}
  ): Promise<SearchResult[]> {
    const opts = { ...DEFAULT_OPTIONS, ...options }

    // Tokenize the query
    const queryTokens = tokenize(query)
    if (queryTokens.length === 0) {
      return []
    }

    // Get unique prefixes for shard loading
    const prefixes = [...new Set(queryTokens.map(tokenPrefix))]

    // Load required shards
    const entries = await this.loader.loadShards(prefixes)

    // De-duplicate entries by nodeId (entries can appear in multiple shards)
    const uniqueEntries = this.deduplicateEntries(entries)

    // Score and filter entries
    const scored = this.scoreEntries(uniqueEntries, queryTokens, opts)

    // Sort by score descending
    scored.sort((a, b) => b.score - a.score)

    // Apply pagination
    return scored.slice(opts.offset, opts.offset + opts.limit)
  }

  /**
   * De-duplicate entries by route + nodeId
   *
   * Entries can appear in multiple shards when they contain tokens
   * with different prefixes. Node IDs are document-scoped, so we need
   * to include the route to uniquely identify entries across the site.
   */
  private deduplicateEntries(entries: SearchEntry[]): SearchEntry[] {
    const seen = new Set<string>()
    const unique: SearchEntry[] = []

    for (const entry of entries) {
      const key = `${entry.route}#${entry.nodeId}`
      if (!seen.has(key)) {
        seen.add(key)
        unique.push(entry)
      }
    }

    return unique
  }

  /**
   * Score entries against query tokens
   */
  private scoreEntries(
    entries: SearchEntry[],
    queryTokens: string[],
    options: Required<SearchOptions>
  ): SearchResult[] {
    const results: SearchResult[] = []

    for (const entry of entries) {
      // Apply filters
      if (!this.matchesFilters(entry, options)) {
        continue
      }

      // Tokenize entry text
      const entryTokens = tokenize(entry.text)
      if (entryTokens.length === 0) {
        continue
      }

      // Calculate score and find highlights
      const { score, highlights } = this.calculateScore(
        entry,
        entryTokens,
        queryTokens
      )

      if (score > 0) {
        results.push({ entry, score, highlights })
      }
    }

    return results
  }

  /**
   * Check if an entry matches the filter options
   */
  private matchesFilters(
    entry: SearchEntry,
    options: Required<SearchOptions>
  ): boolean {
    // Filter by node types
    if (
      options.nodeTypes.length > 0 &&
      !options.nodeTypes.includes(entry.nodeType)
    ) {
      return false
    }

    // Filter by route prefixes
    if (options.routes.length > 0) {
      const matchesRoute = options.routes.some((prefix) =>
        entry.route.startsWith(prefix)
      )
      if (!matchesRoute) {
        return false
      }
    }

    return true
  }

  /**
   * Calculate score for an entry against query tokens
   *
   * Scoring:
   * - Exact token match: 2 points
   * - Prefix match: 1 point
   * - Multiply by structural weight
   * - Multiply by query coverage ratio
   */
  private calculateScore(
    entry: SearchEntry,
    entryTokens: string[],
    queryTokens: string[]
  ): { score: number; highlights: TextHighlight[] } {
    let rawScore = 0
    let matchedQueryTokens = 0
    const highlights: TextHighlight[] = []

    // Build a set of entry tokens for faster lookup
    const entryTokenSet = new Set(entryTokens)

    // Score each query token
    for (const queryToken of queryTokens) {
      // Check for exact match
      if (entryTokenSet.has(queryToken)) {
        rawScore += 2
        matchedQueryTokens++
        // Find highlight positions
        const tokenHighlights = this.findHighlights(entry.text, queryToken)
        highlights.push(...tokenHighlights)
        continue
      }

      // Check for prefix match
      const prefixMatch = entryTokens.find((et) => et.startsWith(queryToken))
      if (prefixMatch) {
        rawScore += 1
        matchedQueryTokens++
        const tokenHighlights = this.findHighlights(entry.text, queryToken)
        highlights.push(...tokenHighlights)
      }
    }

    if (rawScore === 0) {
      return { score: 0, highlights: [] }
    }

    // Apply structural weight (entry.weight is 1-10, normalize to 0.1-1.0)
    const weightMultiplier = entry.weight / 10

    // Apply query coverage ratio (how many query tokens matched)
    const coverageRatio = matchedQueryTokens / queryTokens.length

    // Final score
    const score = rawScore * weightMultiplier * coverageRatio

    // Deduplicate and merge overlapping highlights
    const mergedHighlights = this.mergeHighlights(highlights)

    return { score, highlights: mergedHighlights }
  }

  /**
   * Find highlight positions for a token in text
   *
   * Handles diacritic-folded matches by normalizing both text and token,
   * then mapping positions back to the original text.
   */
  private findHighlights(text: string, token: string): TextHighlight[] {
    const highlights: TextHighlight[] = []

    // Normalize text and build position mapping
    const { normalized, positionMap } = this.normalizeWithPositions(text)
    const normalizedToken = this.normalizeText(token)

    // Search in normalized text
    let pos = 0
    while (pos < normalized.length) {
      const index = normalized.indexOf(normalizedToken, pos)
      if (index === -1) {
        break
      }

      // Map positions back to original text
      const start = positionMap[index]
      const endNormalized = index + normalizedToken.length
      // End position: use the position after the last matched char
      const end =
        endNormalized < positionMap.length
          ? positionMap[endNormalized]
          : text.length

      highlights.push({ start, end })
      pos = index + 1
    }

    return highlights
  }

  /**
   * Normalize text for matching (lowercase, fold diacritics)
   */
  private normalizeText(text: string): string {
    // NFD normalize to separate base characters from combining marks
    const normalized = text.normalize('NFD')
    let result = ''

    for (const char of normalized) {
      const codePoint = char.codePointAt(0) ?? 0
      // Skip combining diacritical marks (U+0300-U+036F)
      if (codePoint >= 0x0300 && codePoint <= 0x036f) {
        continue
      }
      result += char.toLowerCase()
    }

    return result
  }

  /**
   * Normalize text while tracking position mapping back to original
   *
   * Returns normalized text and an array where positionMap[i] gives
   * the original text position (in UTF-16 code units) for normalized position i.
   * This ensures highlights work correctly with String.prototype.slice().
   *
   * IMPORTANT: Both the normalized string and positionMap use UTF-16 code unit
   * indices to match JavaScript's string indexing (indexOf, slice, etc.).
   * We iterate by code points to properly handle astral characters, but we
   * store one position entry per UTF-16 code unit in the output.
   */
  private normalizeWithPositions(text: string): {
    normalized: string
    positionMap: number[]
  } {
    // NFD normalize to separate base characters from combining marks
    const nfdText = text.normalize('NFD')
    let normalized = ''
    const positionMap: number[] = []

    // Track original position in UTF-16 code units (not code points)
    // This is necessary because String.prototype.slice() uses code unit indices
    const nfdToOriginal: number[] = []
    let originalCodeUnitPos = 0
    let nfdCodeUnitPos = 0

    // Build NFD to original position mapping
    // Iterate code points with for..of, but track code unit positions
    for (const origChar of text) {
      const nfdChars = origChar.normalize('NFD')
      // Track each NFD code unit's position back to the original
      for (let i = 0; i < nfdChars.length; i++) {
        nfdToOriginal[nfdCodeUnitPos++] = originalCodeUnitPos
      }
      // origChar.length gives UTF-16 code units (1 for BMP, 2 for astral)
      originalCodeUnitPos += origChar.length
    }

    // Process NFD text by code points (using for...of), skipping combining marks
    // Track UTF-16 position manually since we need it for the position map
    let nfdUtf16Pos = 0
    for (const char of nfdText) {
      const codePoint = char.codePointAt(0) ?? 0

      // Skip combining diacritical marks (U+0300-U+036F)
      if (codePoint >= 0x0300 && codePoint <= 0x036f) {
        nfdUtf16Pos += char.length
        continue
      }

      const lowerChar = char.toLowerCase()
      normalized += lowerChar

      // Add one position entry per UTF-16 code unit in the output
      // This ensures positionMap indices match normalized string indices
      const origPos = nfdToOriginal[nfdUtf16Pos]
      for (let i = 0; i < lowerChar.length; i++) {
        positionMap.push(origPos)
      }

      nfdUtf16Pos += char.length
    }

    return { normalized, positionMap }
  }

  /**
   * Merge overlapping highlights
   */
  private mergeHighlights(highlights: TextHighlight[]): TextHighlight[] {
    if (highlights.length === 0) {
      return []
    }

    // Sort by start position
    const sorted = [...highlights].sort((a, b) => a.start - b.start)

    const merged: TextHighlight[] = [sorted[0]]

    for (let i = 1; i < sorted.length; i++) {
      const current = sorted[i]
      const last = merged[merged.length - 1]

      if (current.start <= last.end) {
        // Overlapping - extend the last highlight
        last.end = Math.max(last.end, current.end)
      } else {
        // Non-overlapping - add new highlight
        merged.push(current)
      }
    }

    return merged
  }

  /**
   * Get search statistics
   */
  getStats(): { totalEntries: number; cachedPrefixes: number } {
    return {
      totalEntries: this.loader.getTotalEntries(),
      ...this.loader.getCacheStats(),
    }
  }

  /**
   * Clear the shard cache
   */
  clearCache(): void {
    this.loader.clearCache()
  }
}
