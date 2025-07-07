import { readFileSync, readdirSync, existsSync } from 'fs'
import { resolve, join } from 'path'

import { Node } from '@stencila/types'
import { format } from 'prettier'
import { describe, it, expect } from 'vitest'

import { encode } from './html'

// Path to the examples/conversion directory
const EXAMPLES_PATH = resolve(__dirname, '../../../examples/conversion')

// Helper to normalize HTML for comparison and make it easier to
// see differences
async function normalizeHtml(html: string): Promise<string> {
  const prettified = await format(html, {
    parser: 'html',
    printWidth: 80,
    tabWidth: 2,
    useTabs: false,
    htmlWhitespaceSensitivity: 'ignore',
    bracketSameLine: false,
  })

  return (
    prettified
      .replace(/\bid="[^"]*"/g, 'id=xxx')
      // Differences in how single quotes are escaped (is at all)
      .replaceAll('&#x27;', "'")
      .replaceAll('&#39;', "'")
      // These are necessary due to differences indenting <pre> elems in the examples
      // and by prettier
      .replace(/<pre>\s*<code>/g, '<pre><code>')
      .replace(/<\/code>\s*<\/pre>/g, '</code></pre>')
      .replace(/<code>\s*<\/code>/g, '<code></code>')
  )
}

// Helper to read JSON file as a Stencila Node
function readJson(path: string): Node {
  const content = readFileSync(path, 'utf-8')
  return JSON.parse(content)
}

// Helper to read HTML file generated from a Node by Rust
function readHtml(path: string): string {
  return readFileSync(path, 'utf-8')
}

// Get all test directories that have both .json and .dom.html files
function getTestCases(): Array<{
  name: string
  jsonPath: string
  htmlPath: string
}> {
  const cases: Array<{ name: string; jsonPath: string; htmlPath: string }> = []

  const dirs = readdirSync(EXAMPLES_PATH, { withFileTypes: true })
    .filter((dirent) => dirent.isDirectory())
    .map((dirent) => dirent.name)

  for (const dir of dirs) {
    // Filter out complex article and chat test cases for now
    if (dir.startsWith('article-') || dir.startsWith('chat')) {
      continue
    }

    const dirPath = join(EXAMPLES_PATH, dir)
    const jsonPath = join(dirPath, `${dir}.json`)
    const htmlPath = join(dirPath, `${dir}.dom.html`)

    if (existsSync(jsonPath) && existsSync(htmlPath)) {
      cases.push({
        name: dir,
        jsonPath,
        htmlPath,
      })
    }
  }

  return cases
}

describe('encode:golden', () => {
  const testCases = getTestCases()

  testCases.forEach(({ name, jsonPath, htmlPath }) => {
    it(`encode:golden:${name}`, async () => {
      const jsonNode = readJson(jsonPath)
      const expectedHtml = await normalizeHtml(readHtml(htmlPath))
      const actualHtml = await normalizeHtml(encode(jsonNode))
      expect(actualHtml).toBe(expectedHtml)
    })
  })
})
