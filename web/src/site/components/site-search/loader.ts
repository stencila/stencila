/**
 * Search index loader
 *
 * Manages loading and caching of search index shards.
 */

import type { SearchEntry, SearchManifest, ShardInfo } from './types'

/**
 * Manages loading and caching of search index shards
 */
export class SearchIndexLoader {
  private manifest: SearchManifest | null = null
  private manifestLoading: Promise<SearchManifest> | null = null
  private shardCache: Map<string, SearchEntry[]> = new Map()
  private loadingShards: Map<string, Promise<SearchEntry[]>> = new Map()
  private basePath: string
  private prefixToShard: Map<string, ShardInfo> = new Map()

  constructor(basePath: string = '/_search') {
    this.basePath = basePath
  }

  /**
   * Load the manifest (call once at initialization)
   *
   * Safe to call concurrently - only one fetch will be made.
   */
  async loadManifest(): Promise<SearchManifest> {
    // Return cached manifest
    if (this.manifest) {
      return this.manifest
    }

    // Return in-flight promise if already loading
    if (this.manifestLoading) {
      return this.manifestLoading
    }

    // Start loading and cache the promise
    this.manifestLoading = this.fetchManifest()

    try {
      const manifest = await this.manifestLoading
      return manifest
    } finally {
      // Clear loading state (but keep manifest cached)
      this.manifestLoading = null
    }
  }

  /**
   * Fetch and process the manifest
   */
  private async fetchManifest(): Promise<SearchManifest> {
    const response = await fetch(`${this.basePath}/manifest.json`)
    if (!response.ok) {
      throw new Error(`Failed to load search manifest: ${response.status}`)
    }

    this.manifest = await response.json()

    // Build prefix-to-shard lookup by extracting prefix from filename
    // e.g., "shards/ab.json" → "ab"
    for (const shard of this.manifest.shards) {
      const prefix = this.extractPrefix(shard.file)
      if (prefix) {
        this.prefixToShard.set(prefix, shard)
      }
    }

    return this.manifest
  }

  /**
   * Extract the token prefix from a shard filename
   *
   * Examples:
   * - "shards/ab.json" → "ab"
   * - "ab.json" → "ab"
   */
  private extractPrefix(filename: string): string | null {
    // Get basename and remove .json extension
    const basename = filename.split('/').pop() ?? filename
    if (basename.endsWith('.json')) {
      return basename.slice(0, -5)
    }
    return null
  }

  /**
   * Check if manifest is loaded
   */
  isLoaded(): boolean {
    return this.manifest !== null
  }

  /**
   * Get total entry count from manifest
   */
  getTotalEntries(): number {
    return this.manifest?.totalEntries ?? 0
  }

  /**
   * Load entries for a specific token prefix
   */
  async loadShard(prefix: string): Promise<SearchEntry[]> {
    // Ensure manifest is loaded
    if (!this.manifest) {
      await this.loadManifest()
    }

    // Check cache first
    if (this.shardCache.has(prefix)) {
      return this.shardCache.get(prefix)!
    }

    // Find shard that contains this prefix
    const shardInfo = this.prefixToShard.get(prefix)
    if (!shardInfo) {
      // No shard for this prefix - return empty
      return []
    }

    // Check if this shard file is already loading (keyed by file, not prefix)
    if (this.loadingShards.has(shardInfo.file)) {
      return this.loadingShards.get(shardInfo.file)!
    }

    // Load the shard (keyed by file to dedupe concurrent loads)
    const loadPromise = this.fetchShard(shardInfo)
    this.loadingShards.set(shardInfo.file, loadPromise)

    try {
      const entries = await loadPromise
      // Cache entries for this prefix
      this.shardCache.set(prefix, entries)
      return entries
    } finally {
      this.loadingShards.delete(shardInfo.file)
    }
  }

  private async fetchShard(info: ShardInfo): Promise<SearchEntry[]> {
    const response = await fetch(`${this.basePath}/${info.file}`)
    if (!response.ok) {
      throw new Error(`Failed to load shard ${info.file}: ${response.status}`)
    }
    return response.json()
  }

  /**
   * Load multiple shards in parallel
   */
  async loadShards(prefixes: string[]): Promise<SearchEntry[]> {
    const uniquePrefixes = [...new Set(prefixes)]
    const shardPromises = uniquePrefixes.map((p) => this.loadShard(p))
    const shardResults = await Promise.all(shardPromises)
    return shardResults.flat()
  }

  /**
   * Clear the shard cache (useful for memory management)
   */
  clearCache(): void {
    this.shardCache.clear()
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): { cachedPrefixes: number; totalShards: number } {
    return {
      cachedPrefixes: this.shardCache.size,
      totalShards: this.manifest?.shards.length ?? 0,
    }
  }
}
