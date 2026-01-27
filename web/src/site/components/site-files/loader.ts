import type { FileEntry } from './types'

/**
 * Manages loading and caching of files index data
 *
 * The files index is organized by directory, with one JSON file per directory.
 * This loader fetches and caches directory listings on demand.
 */
export class FilesIndexLoader {
  private cache: Map<string, FileEntry[]> = new Map()
  private inFlight: Map<string, Promise<FileEntry[]>> = new Map()
  private basePath: string

  constructor(basePath: string = '/_files') {
    this.basePath = basePath
  }

  /**
   * Normalize and encode directory path for safe URL construction.
   * Rejects paths with .. segments, strips leading/trailing slashes.
   *
   * @returns normalized path or null if invalid
   */
  private normalizePath(dir: string): string | null {
    // Strip leading and trailing slashes
    const normalized = dir.replace(/^\/+|\/+$/g, '')

    // Split into segments for validation and encoding
    const segments = normalized.split('/').filter((s) => s.length > 0)

    // Reject path traversal attempts (check for ".." as a segment, not substring)
    if (segments.some((segment) => segment === '..')) {
      console.warn('FilesIndexLoader: Path traversal rejected:', dir)
      return null
    }

    // URL-encode each path segment
    return segments.map((segment) => encodeURIComponent(segment)).join('/')
  }

  /**
   * Load files for a specific directory
   *
   * @param directory - Directory path relative to site root (e.g., "docs/guide")
   * @returns Array of file entries in that directory
   */
  async loadDirectory(directory: string): Promise<FileEntry[]> {
    const normalized = this.normalizePath(directory)
    if (normalized === null) {
      return [] // Invalid path
    }

    const url = normalized
      ? `${this.basePath}/${normalized}.json`
      : `${this.basePath}/_root.json`

    // Check cache (includes negative results)
    if (this.cache.has(url)) {
      return this.cache.get(url)!
    }

    // Check if already fetching
    if (this.inFlight.has(url)) {
      return this.inFlight.get(url)!
    }

    // Start fetch
    const fetchPromise = this.fetchDirectory(url)
    this.inFlight.set(url, fetchPromise)

    try {
      const entries = await fetchPromise
      this.cache.set(url, entries)
      return entries
    } finally {
      this.inFlight.delete(url)
    }
  }

  /**
   * Internal fetch implementation
   */
  private async fetchDirectory(url: string): Promise<FileEntry[]> {
    const response = await fetch(url)

    // 404 means no files in that directory
    if (response.status === 404) {
      return []
    }

    // Other errors should be reported
    if (!response.ok) {
      console.error(`FilesIndexLoader: Failed to load ${url}: ${response.status}`)
      throw new Error(`Failed to load files index: ${response.status}`)
    }

    const entries: FileEntry[] = await response.json()
    return entries
  }

  /**
   * Clear the cache
   *
   * Call this if you know the files index has changed (e.g., after an upload)
   */
  clearCache(): void {
    this.cache.clear()
  }

  /**
   * Clear cache for a specific directory
   */
  clearDirectoryCache(directory: string): void {
    const normalized = this.normalizePath(directory)
    if (normalized === null) return

    const url = normalized
      ? `${this.basePath}/${normalized}.json`
      : `${this.basePath}/_root.json`

    this.cache.delete(url)
  }
}
