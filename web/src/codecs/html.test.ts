import { readFileSync, readdirSync, existsSync } from 'fs'
import { resolve, join } from 'path'

import { Node } from '@stencila/types'
import { describe, it, expect } from 'vitest'

import { encode } from './html'

// Path to the examples/conversion directory
const EXAMPLES_PATH = resolve(__dirname, '../../../examples/conversion')

// Helper to normalize HTML for comparison
function normalizeHtml(html: string): string {
  return (
    html
      // Normalize all id attributes to xxx
      .replace(/\bid="[^"]*"/g, 'id=xxx')
      // Normalize whitespace
      .replace(/\s+/g, ' ')
      .replace(/>\s+</g, '><')
      .trim()
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
    // Filter out complex article-* test cases for now
    if (dir.startsWith('article-')) {
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
    it(`should correctly encode ${name}`, () => {
      const jsonNode = readJson(jsonPath)
      const expectedHtml = normalizeHtml(readHtml(htmlPath))
      const actualHtml = normalizeHtml(encode(jsonNode))
      expect(actualHtml).toBe(expectedHtml)
    })
  })
})
