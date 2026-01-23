/**
 * Tests for the search engine
 */

import { afterAll, beforeAll, describe, expect, test } from 'vitest'

import { SearchEngine } from './engine'

import type { ShardData } from './types'

// Mock fetch for testing - using ShardData format
const mockShardData: ShardData = {
  tokenDefs: [], // No fuzzy data for basic tests
  entries: [
    {
      nodeId: 'hed_1',
      nodeType: 'Heading',
      route: '/docs/guide/',
      text: 'Getting Started with Stencila',
      weight: 8,
      depth: 1,
    },
    {
      nodeId: 'par_1',
      nodeType: 'Paragraph',
      route: '/docs/guide/',
      text: 'This guide will help you get started with Stencila quickly.',
      weight: 1,
      depth: 2,
    },
    {
      nodeId: 'hed_2',
      nodeType: 'Heading',
      route: '/docs/api/',
      text: 'API Reference',
      weight: 8,
      depth: 1,
    },
    {
      nodeId: 'par_2',
      nodeType: 'Paragraph',
      route: '/docs/api/',
      text: 'The Stencila API provides programmatic access to documents.',
      weight: 1,
      depth: 2,
    },
    {
      nodeId: 'dt_1',
      nodeType: 'Datatable',
      route: '/data/',
      text: 'temperature humidity pressure',
      weight: 5,
      depth: 1,
      metadata: {
        columns: ['temperature', 'humidity', 'pressure'],
        rowCount: 100,
      },
    },
  ],
}

const mockManifest = {
  version: 2,
  totalEntries: mockShardData.entries.length,
  totalRoutes: 3,
  shards: {
    ge: { entryCount: 2 },
    st: { entryCount: 3 },
    ap: { entryCount: 1 },
    te: { entryCount: 1 },
  },
}

// Setup fetch mock
const originalFetch = global.fetch
beforeAll(() => {
  global.fetch = async (url: string | URL | Request) => {
    const urlStr = url.toString()
    if (urlStr.endsWith('manifest.json')) {
      return new Response(JSON.stringify(mockManifest), { status: 200 })
    }
    // Return ShardData format for any shard request
    return new Response(JSON.stringify(mockShardData), { status: 200 })
  }
})

afterAll(() => {
  global.fetch = originalFetch
})

describe('SearchEngine', () => {
  test('initializes and loads manifest', async () => {
    const engine = new SearchEngine()
    expect(engine.isReady()).toBe(false)

    await engine.initialize()
    expect(engine.isReady()).toBe(true)
  })

  test('returns empty results for empty query', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('')
    expect(results).toHaveLength(0)

    const results2 = await engine.search('   ')
    expect(results2).toHaveLength(0)
  })

  test('finds matching entries', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('stencila')
    expect(results.length).toBeGreaterThan(0)

    // All results should contain 'stencila' in their text
    for (const result of results) {
      expect(result.entry.text.toLowerCase()).toContain('stencila')
    }
  })

  test('ranks headings higher than paragraphs', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('stencila')

    // Find heading and paragraph results
    const headingResult = results.find((r) => r.entry.nodeType === 'Heading')
    const paragraphResult = results.find(
      (r) => r.entry.nodeType === 'Paragraph'
    )

    if (headingResult && paragraphResult) {
      // Heading should have higher score due to weight
      expect(headingResult.score).toBeGreaterThan(paragraphResult.score)
    }
  })

  test('filters by node type', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('stencila', {
      nodeTypes: ['Heading'],
    })

    // All results should be headings
    for (const result of results) {
      expect(result.entry.nodeType).toBe('Heading')
    }
  })

  test('filters by route prefix', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('stencila', {
      routes: ['/docs/guide/'],
    })

    // All results should be from the guide route
    for (const result of results) {
      expect(result.entry.route).toBe('/docs/guide/')
    }
  })

  test('respects limit option', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('stencila', { limit: 1 })
    expect(results.length).toBeLessThanOrEqual(1)
  })

  test('includes highlights in results', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('stencila')

    // Results should have highlights
    for (const result of results) {
      expect(result.highlights.length).toBeGreaterThan(0)

      // Each highlight should have valid positions
      for (const highlight of result.highlights) {
        expect(highlight.start).toBeGreaterThanOrEqual(0)
        expect(highlight.end).toBeGreaterThan(highlight.start)
        expect(highlight.end).toBeLessThanOrEqual(result.entry.text.length)
      }
    }
  })

  test('handles multi-word queries', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    const results = await engine.search('getting started')
    expect(results.length).toBeGreaterThan(0)

    // Should find the "Getting Started" heading
    const headingResult = results.find((r) =>
      r.entry.text.toLowerCase().includes('getting started')
    )
    expect(headingResult).toBeDefined()
  })

  test('provides stats', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    // Perform a search to populate cache
    await engine.search('test')

    const stats = engine.getStats()
    expect(stats.totalEntries).toBe(mockShardData.entries.length)
    expect(stats.cachedPrefixes).toBeGreaterThan(0)
  })

  test('clears cache', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    // Perform a search to populate cache
    await engine.search('test')
    expect(engine.getStats().cachedPrefixes).toBeGreaterThan(0)

    // Clear cache
    engine.clearCache()
    expect(engine.getStats().cachedPrefixes).toBe(0)
  })

  test('de-duplicates entries from multiple shards', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    // Search for "getting started" - tokens have different prefixes (ge, st)
    // which would load multiple shards, each containing the same entry
    const results = await engine.search('getting started')

    // Count occurrences of each nodeId
    const nodeIdCounts = new Map<string, number>()
    for (const result of results) {
      const count = nodeIdCounts.get(result.entry.nodeId) ?? 0
      nodeIdCounts.set(result.entry.nodeId, count + 1)
    }

    // Each nodeId should appear exactly once (no duplicates)
    for (const [nodeId, count] of nodeIdCounts) {
      expect(count).toBe(1)
    }
  })

  test('highlights work with diacritics', async () => {
    // Create a custom mock with diacritic text
    const diacriticShardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_cafe',
          nodeType: 'Heading',
          route: '/menu/',
          text: 'Welcome to the café',
          weight: 8,
          depth: 1,
        },
        {
          nodeId: 'par_naive',
          nodeType: 'Paragraph',
          route: '/docs/',
          text: 'A naïve approach to the problem',
          weight: 1,
          depth: 2,
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { ca: { entryCount: 1 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(diacriticShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search for "cafe" (without accent) should match "café" (with accent)
      const results = await engine.search('cafe')
      expect(results.length).toBeGreaterThan(0)

      const cafeResult = results.find((r) => r.entry.nodeId === 'hed_cafe')
      expect(cafeResult).toBeDefined()

      // Highlights should exist and point to the correct position
      expect(cafeResult!.highlights.length).toBeGreaterThan(0)

      // The highlight should cover "café" in the original text
      const highlight = cafeResult!.highlights[0]
      const highlightedText = cafeResult!.entry.text.slice(
        highlight.start,
        highlight.end
      )
      expect(highlightedText.toLowerCase()).toBe('café')
    } finally {
      global.fetch = customFetch
    }
  })

  test('highlights map correctly for text with multiple diacritics', async () => {
    const diacriticShardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'par_complex',
          nodeType: 'Paragraph',
          route: '/test/',
          text: 'The résumé was très élégant',
          weight: 1,
          depth: 1,
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { re: { entryCount: 1 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(diacriticShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search for "resume" should match "résumé"
      const results = await engine.search('resume')
      expect(results.length).toBeGreaterThan(0)

      const result = results[0]
      expect(result.highlights.length).toBeGreaterThan(0)

      // Verify highlight extracts the accented word
      const highlight = result.highlights[0]
      const highlightedText = result.entry.text.slice(
        highlight.start,
        highlight.end
      )
      expect(highlightedText).toBe('résumé')
    } finally {
      global.fetch = customFetch
    }
  })

  test('highlights work correctly with emoji (astral characters)', async () => {
    // Emoji are astral characters (outside BMP) represented as surrogate pairs
    // in UTF-16, so they have length 2 in JavaScript strings
    const emojiShardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'par_emoji',
          nodeType: 'Paragraph',
          route: '/test/',
          text: '😀 Welcome to the café! 🎉',
          weight: 1,
          depth: 1,
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { ca: { entryCount: 1 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(emojiShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search for "cafe" should match "café" even with emoji before it
      const results = await engine.search('cafe')
      expect(results.length).toBeGreaterThan(0)

      const result = results[0]
      expect(result.highlights.length).toBeGreaterThan(0)

      // The highlight should correctly extract "café" using slice()
      // This verifies positions are in UTF-16 code units, not code points
      const highlight = result.highlights[0]
      const highlightedText = result.entry.text.slice(
        highlight.start,
        highlight.end
      )
      expect(highlightedText).toBe('café')
    } finally {
      global.fetch = customFetch
    }
  })

  test('highlights work with emoji in the middle of text', async () => {
    const emojiShardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'par_emoji2',
          nodeType: 'Paragraph',
          route: '/test/',
          text: 'Hello 🌍 world 🎉 test',
          weight: 1,
          depth: 1,
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { te: { entryCount: 1 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(emojiShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search for "test" which comes after multiple emoji
      const results = await engine.search('test')
      expect(results.length).toBeGreaterThan(0)

      const result = results[0]
      expect(result.highlights.length).toBeGreaterThan(0)

      // Verify the highlight extracts "test" correctly
      const highlight = result.highlights[0]
      const highlightedText = result.entry.text.slice(
        highlight.start,
        highlight.end
      )
      expect(highlightedText).toBe('test')
    } finally {
      global.fetch = customFetch
    }
  })

  test('fuzzy matching finds entries with typos', async () => {
    // Create ShardData with tokenDefs and compact token references
    const fuzzyShardData: ShardData = {
      tokenDefs: [
        {
          token: 'introduction',
          trigrams: [
            'int',
            'ntr',
            'tro',
            'rod',
            'odu',
            'duc',
            'uct',
            'cti',
            'tio',
            'ion',
          ],
        },
        {
          token: 'database',
          trigrams: ['dat', 'ata', 'tab', 'aba', 'bas', 'ase'],
        },
        {
          token: 'system',
          trigrams: ['sys', 'yst', 'ste', 'tem'],
        },
      ],
      entries: [
        {
          nodeId: 'hed_database',
          nodeType: 'Heading',
          route: '/docs/',
          text: 'Introduction to the database system',
          weight: 8,
          depth: 1,
          // tokens: [[defIndex, start, end], ...]
          tokens: [
            [0, 0, 12], // introduction
            [1, 20, 28], // database
            [2, 29, 35], // system
          ],
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { da: { entryCount: 1 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(fuzzyShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search for "databse" (typo for "database")
      const results = await engine.search('databse')
      expect(results.length).toBeGreaterThan(0)

      // Should find the database entry via fuzzy matching
      const result = results.find((r) => r.entry.nodeId === 'hed_database')
      expect(result).toBeDefined()

      // Highlights should point to the matched token position
      expect(result!.highlights.length).toBeGreaterThan(0)
      const highlight = result!.highlights[0]
      expect(highlight.start).toBe(20) // "database" starts at position 20
      expect(highlight.end).toBe(28) // "database" ends at position 28
    } finally {
      global.fetch = customFetch
    }
  })

  test('exact matches score higher than fuzzy matches', async () => {
    const mixedShardData: ShardData = {
      tokenDefs: [
        {
          token: 'database',
          trigrams: ['dat', 'ata', 'tab', 'aba', 'bas', 'ase'],
        },
        {
          token: 'better',
          trigrams: ['bet', 'ett', 'tte', 'ter'],
        },
      ],
      entries: [
        {
          nodeId: 'par_exact',
          nodeType: 'Paragraph',
          route: '/docs/',
          text: 'The databse is great', // Contains exact "databse"
          weight: 1,
          depth: 1,
          // No tokens needed for exact match
        },
        {
          nodeId: 'par_fuzzy',
          nodeType: 'Paragraph',
          route: '/docs/',
          text: 'The database is better',
          weight: 1,
          depth: 1,
          tokens: [
            [0, 4, 12], // database
            [1, 16, 22], // better
          ],
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { da: { entryCount: 2 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(mixedShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search for "databse" - one entry has exact match, one has fuzzy
      const results = await engine.search('databse')
      expect(results.length).toBeGreaterThanOrEqual(2)

      // Find both results
      const exactResult = results.find((r) => r.entry.nodeId === 'par_exact')
      const fuzzyResult = results.find((r) => r.entry.nodeId === 'par_fuzzy')

      expect(exactResult).toBeDefined()
      expect(fuzzyResult).toBeDefined()

      // Exact match should score higher
      expect(exactResult!.score).toBeGreaterThan(fuzzyResult!.score)
    } finally {
      global.fetch = customFetch
    }
  })

  test('fuzzy matching can be disabled', async () => {
    const fuzzyShardData: ShardData = {
      tokenDefs: [
        {
          token: 'database',
          trigrams: ['dat', 'ata', 'tab', 'aba', 'bas', 'ase'],
        },
      ],
      entries: [
        {
          nodeId: 'hed_test',
          nodeType: 'Heading',
          route: '/docs/',
          text: 'Introduction to the database system',
          weight: 8,
          depth: 1,
          tokens: [[0, 20, 28]], // database
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { da: { entryCount: 1 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(fuzzyShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search with fuzzy disabled - typo should not match
      const results = await engine.search('databse', { enableFuzzy: false })
      expect(results.length).toBe(0)

      // Same search with fuzzy enabled should match
      const resultsWithFuzzy = await engine.search('databse', {
        enableFuzzy: true,
      })
      expect(resultsWithFuzzy.length).toBeGreaterThan(0)
    } finally {
      global.fetch = customFetch
    }
  })

  test('fuzzy threshold controls match sensitivity', async () => {
    const fuzzyShardData: ShardData = {
      tokenDefs: [
        {
          token: 'database',
          trigrams: ['dat', 'ata', 'tab', 'aba', 'bas', 'ase'],
        },
      ],
      entries: [
        {
          nodeId: 'hed_test',
          nodeType: 'Heading',
          route: '/docs/',
          text: 'Working with the database',
          weight: 8,
          depth: 1,
          tokens: [[0, 17, 25]], // database
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: { da: { entryCount: 1 } },
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(fuzzyShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // "databse" vs "database" Jaccard similarity:
      // "databse" trigrams: dat, ata, tab, abs, bse (5 unique)
      // "database" trigrams: dat, ata, tab, aba, bas, ase (6 unique)
      // Intersection: dat, ata, tab (3)
      // Union: 8 unique trigrams
      // Jaccard = 3/8 = 0.375
      //
      // With high threshold (0.5), should not match
      const highThreshold = await engine.search('databse', {
        fuzzyThreshold: 0.5,
      })
      expect(highThreshold.length).toBe(0)

      // With lower threshold (0.3), should match
      const lowThreshold = await engine.search('databse', {
        fuzzyThreshold: 0.3,
      })
      expect(lowThreshold.length).toBeGreaterThan(0)
    } finally {
      global.fetch = customFetch
    }
  })
})
