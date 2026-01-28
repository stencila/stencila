/**
 * Tests for the search engine
 */

import { afterAll, beforeAll, describe, expect, test } from 'vitest'

import { parseQuery, SearchEngine } from './engine'
import type { ShardData } from './types'

// Mock fetch for testing - using ShardData format
const mockShardData: ShardData = {
  tokenDefs: [], // No fuzzy data for basic tests
  entries: [
    {
      nodeId: 'hed_1',
      nodeType: 'Heading',
      route: '/docs/guide/',
      breadcrumbs: ['Home', 'Docs', 'Guide'],
      text: 'Getting Started with Stencila',
      weight: 8,
      depth: 1,
    },
    {
      nodeId: 'par_1',
      nodeType: 'Paragraph',
      route: '/docs/guide/',
      breadcrumbs: ['Home', 'Docs', 'Guide'],
      text: 'This guide will help you get started with Stencila quickly.',
      weight: 1,
      depth: 2,
    },
    {
      nodeId: 'hed_2',
      nodeType: 'Heading',
      route: '/docs/api/',
      breadcrumbs: ['Home', 'Docs', 'API'],
      text: 'API Reference',
      weight: 8,
      depth: 1,
    },
    {
      nodeId: 'par_2',
      nodeType: 'Paragraph',
      route: '/docs/api/',
      breadcrumbs: ['Home', 'Docs', 'API'],
      text: 'The Stencila API provides programmatic access to documents.',
      weight: 1,
      depth: 2,
    },
    {
      nodeId: 'dt_1',
      nodeType: 'Datatable',
      route: '/data/',
      breadcrumbs: ['Home', 'Data'],
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

// Root manifest with access-level sharding (single 'public' level for tests)
const mockManifest = {
  version: 2,
  totalEntries: mockShardData.entries.length,
  totalRoutes: 3,
  levels: {
    public: {
      entryCount: mockShardData.entries.length,
      shardCount: 4,
    },
  },
}

// Per-level manifest for 'public' level
const mockLevelManifest = {
  shards: {
    ge: { entryCount: 2 },
    st: { entryCount: 3 },
    ap: { entryCount: 1 },
    te: { entryCount: 1 },
  },
}

/**
 * Helper to create a fetch mock for access-level sharded search index
 * @param shards - Map of shard prefix to entry count
 * @param shardData - The shard data to return
 * @param entryCount - Optional total entry count (defaults to shardData.entries.length)
 */
function createFetchMock(
  shards: Record<string, { entryCount: number }>,
  shardData: ShardData,
  entryCount?: number,
) {
  const totalEntries = entryCount ?? shardData.entries.length
  const rootManifest = {
    version: 2,
    totalEntries,
    totalRoutes: 1,
    levels: {
      public: {
        entryCount: totalEntries,
        shardCount: Object.keys(shards).length,
      },
    },
  }
  const levelManifest = { shards }

  return async (url: string | URL | Request) => {
    const urlStr = url.toString()
    // Root manifest
    if (urlStr.endsWith('/_search/manifest.json')) {
      return new Response(JSON.stringify(rootManifest), { status: 200 })
    }
    // Level manifest
    if (urlStr.match(/\/_search\/\w+\/manifest\.json$/)) {
      return new Response(JSON.stringify(levelManifest), { status: 200 })
    }
    // Shards
    return new Response(JSON.stringify(shardData), { status: 200 })
  }
}

// Setup fetch mock for access-level sharded index
const originalFetch = global.fetch
beforeAll(() => {
  global.fetch = async (url: string | URL | Request) => {
    const urlStr = url.toString()
    // Root manifest at /_search/manifest.json
    if (urlStr.endsWith('/_search/manifest.json')) {
      return new Response(JSON.stringify(mockManifest), { status: 200 })
    }
    // Level manifest at /_search/{level}/manifest.json
    if (urlStr.match(/\/_search\/\w+\/manifest\.json$/)) {
      return new Response(JSON.stringify(mockLevelManifest), { status: 200 })
    }
    // Shards at /_search/{level}/shards/{prefix}.json
    if (urlStr.match(/\/_search\/\w+\/shards\/\w+\.json$/)) {
      return new Response(JSON.stringify(mockShardData), { status: 200 })
    }
    // Fallback for backward compatibility with any manifest.json
    if (urlStr.endsWith('manifest.json')) {
      return new Response(JSON.stringify(mockManifest), { status: 200 })
    }
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
      (r) => r.entry.nodeType === 'Paragraph',
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
      r.entry.text.toLowerCase().includes('getting started'),
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
    expect(stats.cachedShards).toBeGreaterThan(0)
  })

  test('clears cache', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    // Perform a search to populate cache
    await engine.search('test')
    expect(engine.getStats().cachedShards).toBeGreaterThan(0)

    // Clear cache
    engine.clearCache()
    expect(engine.getStats().cachedShards).toBe(0)
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
    for (const [_nodeId, count] of nodeIdCounts) {
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
          breadcrumbs: ['Home', 'Menu'],
          text: 'Welcome to the café',
          weight: 8,
          depth: 1,
        },
        {
          nodeId: 'par_naive',
          nodeType: 'Paragraph',
          route: '/docs/',
          breadcrumbs: ['Home', 'Docs'],
          text: 'A naïve approach to the problem',
          weight: 1,
          depth: 2,
        },
      ],
    }

    const customFetch = global.fetch
    global.fetch = createFetchMock({ ca: { entryCount: 1 } }, diacriticShardData)

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
        highlight.end,
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
          breadcrumbs: ['Home', 'Test'],
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
          { status: 200 },
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
        highlight.end,
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
          breadcrumbs: ['Home', 'Test'],
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
          { status: 200 },
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
        highlight.end,
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
          breadcrumbs: ['Home', 'Test'],
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
          { status: 200 },
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
        highlight.end,
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
          breadcrumbs: ['Home', 'Docs'],
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
          { status: 200 },
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
          breadcrumbs: ['Home', 'Docs'],
          text: 'The databse is great', // Contains exact "databse"
          weight: 1,
          depth: 1,
          // No tokens needed for exact match
        },
        {
          nodeId: 'par_fuzzy',
          nodeType: 'Paragraph',
          route: '/docs/',
          breadcrumbs: ['Home', 'Docs'],
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
          { status: 200 },
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
          breadcrumbs: ['Home', 'Docs'],
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
          { status: 200 },
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
          breadcrumbs: ['Home', 'Docs'],
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
          { status: 200 },
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

describe('Adjacency Bonus', () => {
  test('adjacent tokens score higher than scattered tokens', async () => {
    // Entry with "getting started" as adjacent phrase
    const adjacentShardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_adjacent',
          nodeType: 'Heading',
          route: '/docs/getting-started/',
          breadcrumbs: ['Home', 'Docs', 'Getting Started'],
          text: 'Getting started with Stencila',
          weight: 8,
          depth: 1,
        },
        {
          nodeId: 'hed_scattered',
          nodeType: 'Heading',
          route: '/docs/other/',
          breadcrumbs: ['Home', 'Docs', 'Other'],
          text: 'Getting help to get started later',
          weight: 8,
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
            shards: { ge: { entryCount: 2 }, st: { entryCount: 2 } },
          }),
          { status: 200 },
        )
      }
      return new Response(JSON.stringify(adjacentShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      const results = await engine.search('getting started')
      expect(results.length).toBe(2)

      // Find both results
      const adjacentResult = results.find(
        (r) => r.entry.nodeId === 'hed_adjacent',
      )
      const scatteredResult = results.find(
        (r) => r.entry.nodeId === 'hed_scattered',
      )

      expect(adjacentResult).toBeDefined()
      expect(scatteredResult).toBeDefined()

      // Adjacent phrase should score higher due to adjacency bonus
      expect(adjacentResult!.score).toBeGreaterThan(scatteredResult!.score)
    } finally {
      global.fetch = customFetch
    }
  })

  test('single word queries receive no adjacency bonus', async () => {
    const singleWordShardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_test',
          nodeType: 'Heading',
          route: '/docs/',
          breadcrumbs: ['Home', 'Docs'],
          text: 'Documentation overview',
          weight: 8,
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
            shards: { do: { entryCount: 1 } },
          }),
          { status: 200 },
        )
      }
      return new Response(JSON.stringify(singleWordShardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Single word query should work without adjacency bonus
      const results = await engine.search('documentation')
      expect(results.length).toBe(1)
      expect(results[0].entry.nodeId).toBe('hed_test')
    } finally {
      global.fetch = customFetch
    }
  })
})

describe('parseQuery', () => {
  test('parses unquoted terms as optional', () => {
    const result = parseQuery('cats and dogs')
    expect(result.terms).toHaveLength(3)
    expect(result.terms.every((t) => !t.required)).toBe(true)
    expect(result.terms.every((t) => !t.adjacentRequired)).toBe(true)
    expect(result.allTokens).toEqual(['cats', 'and', 'dogs'])
  })

  test('parses single quoted word as required', () => {
    const result = parseQuery('"cats"')
    expect(result.terms).toHaveLength(1)
    expect(result.terms[0].required).toBe(true)
    expect(result.terms[0].adjacentRequired).toBe(false)
    expect(result.terms[0].tokens).toEqual(['cats'])
  })

  test('parses multi-word quoted phrase as required with adjacency', () => {
    const result = parseQuery('"getting started"')
    expect(result.terms).toHaveLength(1)
    expect(result.terms[0].required).toBe(true)
    expect(result.terms[0].adjacentRequired).toBe(true)
    expect(result.terms[0].tokens).toEqual(['getting', 'started'])
  })

  test('parses mixed query with required and optional terms', () => {
    const result = parseQuery('"cats" and dogs')
    expect(result.terms).toHaveLength(3)
    // First term is required (quoted)
    expect(result.terms[0].required).toBe(true)
    expect(result.terms[0].tokens).toEqual(['cats'])
    // Remaining terms are optional
    expect(result.terms[1].required).toBe(false)
    expect(result.terms[2].required).toBe(false)
    expect(result.allTokens).toEqual(['cats', 'and', 'dogs'])
  })

  test('parses multiple quoted terms', () => {
    const result = parseQuery('"cats" "dogs"')
    expect(result.terms).toHaveLength(2)
    expect(result.terms[0].required).toBe(true)
    expect(result.terms[0].tokens).toEqual(['cats'])
    expect(result.terms[1].required).toBe(true)
    expect(result.terms[1].tokens).toEqual(['dogs'])
  })

  test('handles unclosed quote by treating as quoted to end', () => {
    const result = parseQuery('"cats and dogs')
    expect(result.terms).toHaveLength(1)
    expect(result.terms[0].required).toBe(true)
    expect(result.terms[0].adjacentRequired).toBe(true) // Multi-word
    expect(result.terms[0].tokens).toEqual(['cats', 'and', 'dogs'])
  })

  test('handles empty quotes', () => {
    const result = parseQuery('""')
    expect(result.terms).toHaveLength(0)
    expect(result.allTokens).toHaveLength(0)
  })

  test('handles adjacent quotes', () => {
    const result = parseQuery('"hello""world"')
    expect(result.terms).toHaveLength(2)
    expect(result.terms[0].required).toBe(true)
    expect(result.terms[0].tokens).toEqual(['hello'])
    expect(result.terms[1].required).toBe(true)
    expect(result.terms[1].tokens).toEqual(['world'])
  })

  test('normalizes quoted terms (case insensitive, diacritics folded)', () => {
    const result = parseQuery('"Café"')
    expect(result.terms).toHaveLength(1)
    expect(result.terms[0].tokens).toEqual(['cafe'])
  })

  test('handles empty query', () => {
    const result = parseQuery('')
    expect(result.terms).toHaveLength(0)
    expect(result.allTokens).toHaveLength(0)
  })

  test('handles query with only whitespace', () => {
    const result = parseQuery('   ')
    expect(result.terms).toHaveLength(0)
    expect(result.allTokens).toHaveLength(0)
  })
})

describe('Quoted Term Filtering', () => {
  test('required term filters out entries without it', async () => {
    const shardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_cats',
          nodeType: 'Heading',
          route: '/cats/',
          breadcrumbs: ['Home', 'Cats'],
          text: 'All about cats',
          weight: 8,
          depth: 1,
        },
        {
          nodeId: 'hed_dogs',
          nodeType: 'Heading',
          route: '/dogs/',
          breadcrumbs: ['Home', 'Dogs'],
          text: 'All about dogs',
          weight: 8,
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
            shards: { ca: { entryCount: 1 }, do: { entryCount: 1 } },
          }),
          { status: 200 },
        )
      }
      return new Response(JSON.stringify(shardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Search with quoted "cats" - should only return the cats entry
      const results = await engine.search('"cats"')
      expect(results.length).toBe(1)
      expect(results[0].entry.nodeId).toBe('hed_cats')
    } finally {
      global.fetch = customFetch
    }
  })

  test('quoted phrase requires adjacency', async () => {
    const shardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_adjacent',
          nodeType: 'Heading',
          route: '/adjacent/',
          breadcrumbs: ['Home', 'Adjacent'],
          text: 'Getting started with coding',
          weight: 8,
          depth: 1,
        },
        {
          nodeId: 'hed_scattered',
          nodeType: 'Heading',
          route: '/scattered/',
          breadcrumbs: ['Home', 'Scattered'],
          text: 'Getting help to get started later',
          weight: 8,
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
            shards: { ge: { entryCount: 2 }, st: { entryCount: 2 } },
          }),
          { status: 200 },
        )
      }
      return new Response(JSON.stringify(shardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // Quoted phrase requires adjacency - only the adjacent entry matches
      const results = await engine.search('"getting started"')
      expect(results.length).toBe(1)
      expect(results[0].entry.nodeId).toBe('hed_adjacent')
    } finally {
      global.fetch = customFetch
    }
  })

  test('quoted phrase does not match substrings within larger words', async () => {
    // Regression test: "getting started" should NOT match "forgetting started"
    // because "getting" inside "forgetting" is not a separate token
    const shardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_correct',
          nodeType: 'Heading',
          route: '/correct/',
          breadcrumbs: ['Home', 'Correct'],
          text: 'Getting started with coding',
          weight: 8,
          depth: 1,
        },
        {
          nodeId: 'hed_substring',
          nodeType: 'Heading',
          route: '/substring/',
          breadcrumbs: ['Home', 'Substring'],
          text: 'Forgetting started tasks',
          weight: 8,
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
            shards: { ge: { entryCount: 2 }, st: { entryCount: 2 } },
          }),
          { status: 200 },
        )
      }
      return new Response(JSON.stringify(shardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // "getting started" should only match the first entry
      // It should NOT match "forgetting started" even though "getting" is a substring
      const results = await engine.search('"getting started"')
      expect(results.length).toBe(1)
      expect(results[0].entry.nodeId).toBe('hed_correct')
    } finally {
      global.fetch = customFetch
    }
  })

  test('optional terms boost but do not filter', async () => {
    const shardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_both',
          nodeType: 'Heading',
          route: '/both/',
          breadcrumbs: ['Home', 'Both'],
          text: 'Cats and dogs together',
          weight: 8,
          depth: 1,
        },
        {
          nodeId: 'hed_cats_only',
          nodeType: 'Heading',
          route: '/cats/',
          breadcrumbs: ['Home', 'Cats'],
          text: 'Just cats here',
          weight: 8,
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
            shards: { ca: { entryCount: 2 }, do: { entryCount: 1 } },
          }),
          { status: 200 },
        )
      }
      return new Response(JSON.stringify(shardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // "cats" is required, "dogs" is optional boost
      const results = await engine.search('"cats" dogs')
      expect(results.length).toBe(2) // Both entries have "cats"

      // Entry with both "cats" and "dogs" should score higher
      expect(results[0].entry.nodeId).toBe('hed_both')
      expect(results[1].entry.nodeId).toBe('hed_cats_only')
      expect(results[0].score).toBeGreaterThan(results[1].score)
    } finally {
      global.fetch = customFetch
    }
  })

  test('all-required query has coverage 1.0 (no NaN)', async () => {
    const shardData: ShardData = {
      tokenDefs: [],
      entries: [
        {
          nodeId: 'hed_both',
          nodeType: 'Heading',
          route: '/both/',
          breadcrumbs: ['Home', 'Both'],
          text: 'Cats and dogs',
          weight: 8,
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
            shards: { ca: { entryCount: 1 }, do: { entryCount: 1 } },
          }),
          { status: 200 },
        )
      }
      return new Response(JSON.stringify(shardData), { status: 200 })
    }

    try {
      const engine = new SearchEngine()
      await engine.initialize()

      // All required terms - coverage should be 1.0, not NaN
      const results = await engine.search('"cats" "dogs"')
      expect(results.length).toBe(1)
      expect(Number.isNaN(results[0].score)).toBe(false)
      expect(results[0].score).toBeGreaterThan(0)
    } finally {
      global.fetch = customFetch
    }
  })

  test('backward compatibility: unquoted query works as before', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    // Unquoted queries should work exactly as before
    const results = await engine.search('getting started')
    expect(results.length).toBeGreaterThan(0)

    // Should find entries with both tokens (not require adjacency)
    const headingResult = results.find((r) =>
      r.entry.text.toLowerCase().includes('getting started'),
    )
    expect(headingResult).toBeDefined()
  })

  test('required term that does not exist returns no results', async () => {
    const engine = new SearchEngine()
    await engine.initialize()

    // Search for a term that doesn't exist in any entry
    const results = await engine.search('"nonexistentterm12345"')
    expect(results.length).toBe(0)
  })
})
