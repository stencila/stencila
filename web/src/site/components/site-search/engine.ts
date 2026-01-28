/**
 * Search engine for querying the search index
 *
 * Handles tokenization, shard loading, scoring, and result ranking.
 */

import { SearchIndexLoader } from './loader'
import { generateTrigrams, tokenize, tokenPrefix } from './tokenize'
import type {
  ParsedQuery,
  QueryTerm,
  SearchEntry,
  SearchOptions,
  SearchResult,
  TextHighlight,
  TokenTrigrams,
} from './types'

/**
 * Default search options
 */
const DEFAULT_OPTIONS: Required<SearchOptions> = {
  limit: 20,
  offset: 0,
  nodeTypes: [],
  routes: [],
  enableFuzzy: true,
  fuzzyThreshold: 0.3,
}

/**
 * Adjacency scoring constants
 */
/** Maximum gap (in UTF-16 code units) between tokens to be considered adjacent */
const ADJACENCY_GAP_THRESHOLD = 5

/** Bonus points for each pair of adjacent query tokens in a contiguous chain */
const ADJACENCY_BONUS = 1.5

/** Max query tokens to consider for adjacency (caps exponential complexity) */
const MAX_ADJACENCY_TOKENS = 6

/** Max positions per token to consider (caps exponential complexity) */
const MAX_POSITIONS_PER_TOKEN = 5

/** Max unique prefix-matching tokens to consider (prevents hot spot on common prefixes) */
const MAX_PREFIX_MATCHES = 3

/**
 * Calculate Jaccard similarity between two sets of trigrams
 *
 * Returns a value between 0 and 1, where 1 means identical sets.
 * Returns 0 if either set is empty (guard against division by zero).
 */
function jaccardSimilarity(a: Set<string>, b: Set<string>): number {
  // Guard: if either set is empty, return 0 (no similarity)
  if (a.size === 0 || b.size === 0) {
    return 0
  }

  // Calculate intersection
  let intersectionSize = 0
  for (const item of a) {
    if (b.has(item)) {
      intersectionSize++
    }
  }

  // Union size = |A| + |B| - |A ∩ B|
  const unionSize = a.size + b.size - intersectionSize

  return intersectionSize / unionSize
}

/**
 * Parse a query string into required and optional terms
 *
 * Quoted terms (e.g., "cats" or "getting started") are required - results must
 * contain those exact normalized tokens. Multi-word quoted phrases require adjacency.
 *
 * Unquoted terms are optional boosters - they improve score but don't filter results.
 *
 * Unclosed quotes are treated as quoted to end of string.
 *
 * @param query - The raw query string
 * @returns ParsedQuery with terms and flat token list
 */
export function parseQuery(query: string): ParsedQuery {
  const terms: QueryTerm[] = []
  const allTokens: string[] = []

  let inQuote = false
  let currentSegment = ''

  // Character-by-character parsing to handle edge cases
  for (const char of query) {
    if (char === '"') {
      // Finalize the current segment
      if (currentSegment.length > 0) {
        const tokens = tokenize(currentSegment)
        if (tokens.length > 0) {
          if (inQuote) {
            // Quoted segment - required, adjacent if multi-word
            terms.push({
              tokens,
              required: true,
              adjacentRequired: tokens.length > 1,
            })
          } else {
            // Unquoted segment - each token is optional
            for (const token of tokens) {
              terms.push({
                tokens: [token],
                required: false,
                adjacentRequired: false,
              })
            }
          }
          allTokens.push(...tokens)
        }
        currentSegment = ''
      }
      // Toggle quote state
      inQuote = !inQuote
    } else {
      currentSegment += char
    }
  }

  // Handle remaining segment (including unclosed quotes - treat as quoted)
  if (currentSegment.length > 0) {
    const tokens = tokenize(currentSegment)
    if (tokens.length > 0) {
      if (inQuote) {
        // Unclosed quote - treat as quoted phrase to end
        terms.push({
          tokens,
          required: true,
          adjacentRequired: tokens.length > 1,
        })
      } else {
        // Unquoted segment - each token is optional
        for (const token of tokens) {
          terms.push({
            tokens: [token],
            required: false,
            adjacentRequired: false,
          })
        }
      }
      allTokens.push(...tokens)
    }
  }

  return { terms, allTokens }
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
    await this.loader.loadRootManifest()
  }

  /**
   * Check if the engine is initialized
   */
  isReady(): boolean {
    return this.loader.isLoaded()
  }

  /**
   * Set the access levels the user can access
   *
   * Call this with the user's accessible levels from auth status.
   * Example: For a team member, pass ['public', 'subscriber', 'password', 'team']
   */
  setAccessibleLevels(
    levels: import('../site-action/types').AccessLevel[],
  ): void {
    this.loader.setAccessibleLevels(levels)
  }

  /**
   * Get the currently configured accessible levels
   */
  getAccessibleLevels(): import('../site-action/types').AccessLevel[] {
    return this.loader.getAccessibleLevels()
  }

  /**
   * Execute a search query
   *
   * Supports quoted terms for exact/required matching:
   * - "cats" - requires entries to contain the token "cats"
   * - "getting started" - requires adjacent tokens "getting" and "started"
   * - Unquoted terms boost score but don't filter
   */
  async search(
    query: string,
    options: SearchOptions = {},
  ): Promise<SearchResult[]> {
    const opts = { ...DEFAULT_OPTIONS, ...options }

    // Parse the query into required and optional terms
    const parsedQuery = parseQuery(query)
    if (parsedQuery.allTokens.length === 0) {
      return []
    }

    // Get unique prefixes for shard loading
    const prefixes = [...new Set(parsedQuery.allTokens.map(tokenPrefix))]

    // Load required shards
    const entries = await this.loader.loadShards(prefixes)

    // De-duplicate entries by nodeId (entries can appear in multiple shards)
    const uniqueEntries = this.deduplicateEntries(entries)

    // Score and filter entries
    const scored = this.scoreEntries(uniqueEntries, parsedQuery, opts)

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
   * Score entries against parsed query
   */
  private scoreEntries(
    entries: SearchEntry[],
    parsedQuery: ParsedQuery,
    options: Required<SearchOptions>,
  ): SearchResult[] {
    const results: SearchResult[] = []

    for (const entry of entries) {
      // Apply filters
      if (!this.matchesFilters(entry, options)) {
        continue
      }

      // Get or compute tokenized entry text (lazy cache)
      const entryTokens =
        entry._cachedTokens ?? (entry._cachedTokens = tokenize(entry.text))
      if (entryTokens.length === 0) {
        continue
      }

      // Calculate score and find highlights
      // Returns null if required terms are not satisfied
      const result = this.calculateScore(
        entry,
        entryTokens,
        parsedQuery,
        options,
      )

      if (result !== null && result.score > 0) {
        results.push({
          entry,
          score: result.score,
          highlights: result.highlights,
        })
      }
    }

    return results
  }

  /**
   * Check if an entry matches the filter options
   */
  private matchesFilters(
    entry: SearchEntry,
    options: Required<SearchOptions>,
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
        entry.route.startsWith(prefix),
      )
      if (!matchesRoute) {
        return false
      }
    }

    return true
  }

  /**
   * Calculate score for an entry against a parsed query
   *
   * Required terms (quoted) must all be satisfied or the entry is filtered out (returns null).
   * For multi-word quoted phrases, tokens must also be adjacent.
   *
   * Optional terms (unquoted) boost the score but never filter out entries.
   *
   * Scoring:
   * - Exact token match: 2 points
   * - Prefix match: 1 point (optional terms only)
   * - Fuzzy match (if enabled): 0.3 × similarity (optional terms only)
   * - Adjacency bonus: 1.5 points per adjacent pair in longest contiguous chain
   * - Multiply by structural weight
   * - Multiply by optional query coverage ratio
   */
  private calculateScore(
    entry: SearchEntry,
    entryTokens: string[],
    parsedQuery: ParsedQuery,
    options: Required<SearchOptions>,
  ): { score: number; highlights: TextHighlight[] } | null {
    // Build a set of entry tokens for faster lookup
    const entryTokenSet = new Set(entryTokens)

    // First pass: check all required terms are satisfied
    // Required terms need exact matches only (no prefix/fuzzy)
    for (const term of parsedQuery.terms) {
      if (!term.required) continue

      // All tokens in the term must be present
      for (const token of term.tokens) {
        if (!entryTokenSet.has(token)) {
          return null // Required token missing - filter out
        }
      }

      // For adjacent-required terms, check adjacency using token indices
      // This ensures we match whole tokens, not substrings within larger words
      if (term.adjacentRequired && term.tokens.length > 1) {
        if (!this.checkAdjacency(entryTokens, term.tokens)) {
          return null // Required adjacency not satisfied - filter out
        }
      }
    }

    // Second pass: calculate score for all tokens
    let rawScore = 0
    let matchedOptionalTokens = 0
    let totalOptionalTokens = 0
    const highlights: TextHighlight[] = []

    // Track ALL match positions per query token index DURING matching
    const positionsByQuery: Map<number, { start: number; end: number }[]> =
      new Map()

    // Check if fuzzy matching is available and enabled
    const canFuzzy =
      options.enableFuzzy &&
      entry.tokenTrigrams &&
      entry.tokenTrigrams.length > 0

    // Process all tokens in order (for adjacency tracking)
    let tokenIndex = 0
    for (const term of parsedQuery.terms) {
      const isRequired = term.required

      for (const queryToken of term.tokens) {
        if (!isRequired) {
          totalOptionalTokens++
        }

        // Check for exact match
        if (entryTokenSet.has(queryToken)) {
          rawScore += 2
          if (!isRequired) matchedOptionalTokens++

          // Find highlight positions
          const tokenHighlights = this.findHighlights(entry.text, queryToken)
          highlights.push(...tokenHighlights)
          // Track positions for adjacency check
          positionsByQuery.set(
            tokenIndex,
            tokenHighlights
              .map((h) => ({ start: h.start, end: h.end }))
              .slice(0, MAX_POSITIONS_PER_TOKEN),
          )
          tokenIndex++
          continue
        }

        // For required terms, we already verified exact match above
        // Only try prefix/fuzzy for optional terms
        if (isRequired) {
          tokenIndex++
          continue
        }

        // Check for prefix match - find unique matching tokens with early exit
        const prefixMatches: string[] = []
        const seenPrefixTokens = new Set<string>()
        for (const et of entryTokens) {
          if (et.startsWith(queryToken) && !seenPrefixTokens.has(et)) {
            seenPrefixTokens.add(et)
            prefixMatches.push(et)
            if (prefixMatches.length >= MAX_PREFIX_MATCHES) break
          }
        }

        if (prefixMatches.length > 0) {
          rawScore += 1
          matchedOptionalTokens++

          // For UI: highlight only the query prefix
          const prefixHighlights = this.findHighlights(entry.text, queryToken)
          highlights.push(...prefixHighlights)

          // For adjacency: use full token positions
          const allPositions: { start: number; end: number }[] = []
          for (const match of prefixMatches) {
            const tokenPositions = this.findHighlights(entry.text, match)
            allPositions.push(
              ...tokenPositions.map((h) => ({ start: h.start, end: h.end })),
            )
          }
          positionsByQuery.set(
            tokenIndex,
            allPositions.slice(0, MAX_POSITIONS_PER_TOKEN),
          )
          tokenIndex++
          continue
        }

        // Try fuzzy matching if available
        if (canFuzzy) {
          const fuzzyResult = this.tryFuzzyMatch(
            queryToken,
            entry.tokenTrigrams!,
            options.fuzzyThreshold,
          )
          if (fuzzyResult) {
            rawScore += 0.3 * fuzzyResult.similarity
            matchedOptionalTokens++
            const pos = {
              start: fuzzyResult.matchedToken.start,
              end: fuzzyResult.matchedToken.end,
            }
            highlights.push(pos)
            positionsByQuery.set(tokenIndex, [pos])
          }
        }
        tokenIndex++
      }
    }

    if (rawScore === 0) {
      return { score: 0, highlights: [] }
    }

    // Calculate adjacency bonus for phrase matching
    const adjacencyBonus = this.calculateAdjacencyBonus(
      parsedQuery.allTokens.length,
      positionsByQuery,
    )
    rawScore += adjacencyBonus

    // Apply structural weight (entry.weight is 1-10, normalize to 0.1-1.0)
    const weightMultiplier = entry.weight / 10

    // Apply coverage ratio based on query type
    // If there are required terms, they're already enforced by filtering.
    // Optional terms should only boost, never penalize to zero.
    const hasRequiredTerms = parsedQuery.terms.some((t) => t.required)
    const coverageRatio = hasRequiredTerms
      ? 1.0 // Required terms enforce filtering; optional just boosts
      : totalOptionalTokens > 0
        ? matchedOptionalTokens / totalOptionalTokens
        : 1.0

    // Final score
    const score = rawScore * weightMultiplier * coverageRatio

    // Deduplicate and merge overlapping highlights
    const mergedHighlights = this.mergeHighlights(highlights)

    return { score, highlights: mergedHighlights }
  }

  /**
   * Check if tokens appear adjacent in text (for quoted phrase matching)
   *
   * Uses token-index-based adjacency checking to ensure we're matching
   * whole tokens, not substrings within larger words. This prevents
   * "getting started" from matching "forgetting started".
   *
   * Tokens must be strictly adjacent (consecutive indices in the token array).
   */
  private checkAdjacency(
    entryTokens: string[],
    requiredTokens: string[],
  ): boolean {
    if (requiredTokens.length < 2) return true

    // Build a map of token -> list of indices where it appears
    const tokenIndices = new Map<string, number[]>()
    for (let i = 0; i < entryTokens.length; i++) {
      const token = entryTokens[i]
      const indices = tokenIndices.get(token) ?? []
      indices.push(i)
      tokenIndices.set(token, indices)
    }

    // Check if required tokens can form a strictly adjacent sequence
    // Cap the number of starting positions to prevent exponential blowup
    const firstTokenIndices = tokenIndices.get(requiredTokens[0]) ?? []
    const cappedFirstIndices = firstTokenIndices.slice(
      0,
      MAX_POSITIONS_PER_TOKEN,
    )

    for (const startIdx of cappedFirstIndices) {
      if (
        this.hasAdjacentTokenSequence(tokenIndices, requiredTokens, 1, startIdx)
      ) {
        return true
      }
    }

    return false
  }

  /**
   * Recursively check if tokens form a strictly adjacent sequence (consecutive indices)
   */
  private hasAdjacentTokenSequence(
    tokenIndices: Map<string, number[]>,
    requiredTokens: string[],
    reqIdx: number,
    prevIndex: number,
  ): boolean {
    if (reqIdx >= requiredTokens.length) {
      return true // All tokens matched adjacently
    }

    const nextToken = requiredTokens[reqIdx]
    const nextIndices = tokenIndices.get(nextToken) ?? []

    // The next token must be at exactly prevIndex + 1 (strictly adjacent)
    const expectedIndex = prevIndex + 1
    if (nextIndices.includes(expectedIndex)) {
      return this.hasAdjacentTokenSequence(
        tokenIndices,
        requiredTokens,
        reqIdx + 1,
        expectedIndex,
      )
    }

    return false
  }

  /**
   * Try to find a fuzzy match for a query token against entry tokens
   *
   * Returns the best matching token and its similarity if above threshold,
   * or null if no match meets the threshold.
   */
  private tryFuzzyMatch(
    queryToken: string,
    tokenTrigrams: TokenTrigrams[],
    threshold: number,
  ): { matchedToken: TokenTrigrams; similarity: number } | null {
    // Generate trigrams for the query token
    const queryTrigrams = new Set(generateTrigrams(queryToken))

    // If query token is too short for trigrams, no fuzzy match possible
    if (queryTrigrams.size === 0) {
      return null
    }

    let bestMatch: TokenTrigrams | null = null
    let bestSimilarity = 0

    // Find the best matching token
    for (const tt of tokenTrigrams) {
      const entryTrigrams = new Set(tt.trigrams)
      const similarity = jaccardSimilarity(queryTrigrams, entryTrigrams)

      if (similarity >= threshold && similarity > bestSimilarity) {
        bestMatch = tt
        bestSimilarity = similarity
      }
    }

    if (bestMatch) {
      return { matchedToken: bestMatch, similarity: bestSimilarity }
    }

    return null
  }

  /**
   * Calculate adjacency bonus by finding the longest contiguous chain
   *
   * Tries starting from EVERY query token index (not just 0), so "brown fox"
   * in query "quick brown fox" still gets adjacency bonus.
   *
   * Uses recursive search (not greedy) to find the optimal path when
   * multiple occurrences exist.
   *
   * Complexity is capped by MAX_ADJACENCY_TOKENS (chain length) and MAX_POSITIONS_PER_TOKEN
   * to prevent exponential blowup on long queries or common tokens.
   */
  private calculateAdjacencyBonus(
    queryTokenCount: number,
    positionsByQuery: Map<number, { start: number; end: number }[]>,
  ): number {
    if (queryTokenCount < 2) return 0

    let maxChainLength = 0

    // Try starting a chain from EACH query token index (all tokens, not just first N)
    // This allows phrases at the END of long queries to get adjacency bonus
    for (let startIdx = 0; startIdx < queryTokenCount; startIdx++) {
      const startPositions = positionsByQuery.get(startIdx) ?? []

      for (const startPos of startPositions) {
        // Recursively find the longest chain from this starting point
        // Chain length is capped by MAX_ADJACENCY_TOKENS, not starting index
        const chainLength = this.findLongestChain(
          startIdx,
          startPos.end,
          queryTokenCount,
          positionsByQuery,
          1, // Start with chain length of 1 (current token)
        )
        maxChainLength = Math.max(maxChainLength, chainLength)
      }
    }

    // Chain of length N has (N-1) adjacent pairs
    return Math.max(0, maxChainLength - 1) * ADJACENCY_BONUS
  }

  /**
   * Recursively find the longest chain starting at queryIdx with previous token ending at prevEnd
   *
   * Tries ALL adjacent positions for the next token (not just the first),
   * ensuring we don't miss longer chains due to greedy selection.
   *
   * Chain length is capped at MAX_ADJACENCY_TOKENS to bound complexity.
   */
  private findLongestChain(
    queryIdx: number,
    prevEnd: number,
    queryTokenCount: number,
    positionsByQuery: Map<number, { start: number; end: number }[]>,
    currentChainLength: number,
  ): number {
    // Base case: no more tokens to chain, or chain length cap reached
    if (
      queryIdx + 1 >= queryTokenCount ||
      currentChainLength >= MAX_ADJACENCY_TOKENS
    ) {
      return currentChainLength
    }

    const nextPositions = positionsByQuery.get(queryIdx + 1) ?? []
    let maxChainLength = currentChainLength

    // Try ALL adjacent positions, not just the first (non-greedy)
    for (const nextPos of nextPositions) {
      const gap = nextPos.start - prevEnd
      if (gap >= 0 && gap <= ADJACENCY_GAP_THRESHOLD) {
        const chainLength = this.findLongestChain(
          queryIdx + 1,
          nextPos.end,
          queryTokenCount,
          positionsByQuery,
          currentChainLength + 1,
        )
        maxChainLength = Math.max(maxChainLength, chainLength)
      }
    }

    return maxChainLength
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
    normalized: string;
    positionMap: number[];
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
  getStats(): {
    totalEntries: number;
    cachedShards: number;
    cachedLevels: number;
    totalShards: number;
  } {
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
