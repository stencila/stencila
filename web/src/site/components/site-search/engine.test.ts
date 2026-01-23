/**
 * Tests for the search engine
 */

import { afterAll, beforeAll, describe, expect, test } from 'vitest'

import { SearchEngine } from './engine'

// Mock fetch for testing
const mockEntries = [
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
]

const mockManifest = {
  version: 1,
  totalEntries: mockEntries.length,
  totalRoutes: 3,
  shards: [
    { file: 'shards/ge.json', entryCount: 2 },
    { file: 'shards/st.json', entryCount: 3 },
    { file: 'shards/ap.json', entryCount: 1 },
    { file: 'shards/te.json', entryCount: 1 },
  ],
}

// Setup fetch mock
const originalFetch = global.fetch
beforeAll(() => {
  global.fetch = async (url: string | URL | Request) => {
    const urlStr = url.toString()
    if (urlStr.endsWith('manifest.json')) {
      return new Response(JSON.stringify(mockManifest), { status: 200 })
    }
    // Return all entries for any shard request (simplified for testing)
    return new Response(JSON.stringify(mockEntries), { status: 200 })
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
    expect(stats.totalEntries).toBe(mockEntries.length)
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
    const diacriticEntries = [
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
    ]

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: [{ file: 'shards/ca.json', entryCount: 1 }],
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(diacriticEntries), { status: 200 })
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
    const diacriticEntries = [
      {
        nodeId: 'par_complex',
        nodeType: 'Paragraph',
        route: '/test/',
        text: 'The résumé was très élégant',
        weight: 1,
        depth: 1,
      },
    ]

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: [{ file: 'shards/re.json', entryCount: 1 }],
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(diacriticEntries), { status: 200 })
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
    const emojiEntries = [
      {
        nodeId: 'par_emoji',
        nodeType: 'Paragraph',
        route: '/test/',
        text: '😀 Welcome to the café! 🎉',
        weight: 1,
        depth: 1,
      },
    ]

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: [{ file: 'shards/ca.json', entryCount: 1 }],
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(emojiEntries), { status: 200 })
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
    const emojiEntries = [
      {
        nodeId: 'par_emoji2',
        nodeType: 'Paragraph',
        route: '/test/',
        text: 'Hello 🌍 world 🎉 test',
        weight: 1,
        depth: 1,
      },
    ]

    const customFetch = global.fetch
    global.fetch = async (url: string | URL | Request) => {
      const urlStr = url.toString()
      if (urlStr.endsWith('manifest.json')) {
        return new Response(
          JSON.stringify({
            ...mockManifest,
            shards: [{ file: 'shards/te.json', entryCount: 1 }],
          }),
          { status: 200 }
        )
      }
      return new Response(JSON.stringify(emojiEntries), { status: 200 })
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
})
