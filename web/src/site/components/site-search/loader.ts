/**
 * Search index loader
 *
 * Manages loading and caching of search index shards.
 */

import type { SearchEntry, SearchManifest, ShardData } from './types'

/**
 * Manages loading and caching of search index shards
 */
export class SearchIndexLoader {
  private manifest: SearchManifest | null = null
  private manifestLoading: Promise<SearchManifest> | null = null
  private shardCache: Map<string, SearchEntry[]> = new Map()
  private loadingShards: Map<string, Promise<SearchEntry[]>> = new Map()
  private basePath: string

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
    return this.manifest
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

    // Check if shard exists for this prefix (O(1) lookup)
    if (!this.manifest!.shards[prefix]) {
      // No shard for this prefix - return empty
      return []
    }

    // Check if this shard is already loading
    if (this.loadingShards.has(prefix)) {
      return this.loadingShards.get(prefix)!
    }

    // Load the shard
    const loadPromise = this.fetchShard(prefix)
    this.loadingShards.set(prefix, loadPromise)

    try {
      const entries = await loadPromise
      // Cache entries for this prefix
      this.shardCache.set(prefix, entries)
      return entries
    } finally {
      this.loadingShards.delete(prefix)
    }
  }

  private async fetchShard(prefix: string): Promise<SearchEntry[]> {
    const response = await fetch(`${this.basePath}/shards/${prefix}.json`)
    if (!response.ok) {
      throw new Error(`Failed to load shard ${prefix}: ${response.status}`)
    }
    const data: unknown = await response.json()
    const shardData = this.validateShardData(data, prefix)
    return this.expandTokens(shardData)
  }

  /**
   * Validate that loaded data has the expected ShardData shape
   *
   * Fails fast with a clear error if the shard format is invalid,
   * which can happen with stale cached shards or partial writes.
   */
  private validateShardData(data: unknown, prefix: string): ShardData {
    if (!data || typeof data !== 'object' || Array.isArray(data)) {
      throw new Error(
        `Shard ${prefix}: expected ShardData object, got ${Array.isArray(data) ? 'array (possibly old format)' : typeof data}`,
      )
    }
    const shard = data as Record<string, unknown>
    if (!Array.isArray(shard.entries)) {
      throw new Error(`Shard ${prefix}: missing or invalid 'entries' array`)
    }
    if (!Array.isArray(shard.tokenDefs)) {
      throw new Error(`Shard ${prefix}: missing or invalid 'tokenDefs' array`)
    }
    return data as ShardData
  }

  /**
   * Expand compact token references to full TokenTrigrams
   *
   * Converts the compact [defIndex, start, end] tuples in each entry
   * to full TokenTrigrams objects using the shard's tokenDefs array.
   * Skips invalid token refs (out of bounds) to handle stale/corrupted data.
   * Deletes the compact form after expansion to reduce memory usage.
   */
  private expandTokens(data: ShardData): SearchEntry[] {
    const tokenDefsLength = data.tokenDefs.length

    for (const entry of data.entries) {
      if (entry.tokens && tokenDefsLength > 0) {
        // Filter out invalid token refs (idx out of bounds)
        const validTokens = entry.tokens.filter(
          ([idx]) => idx >= 0 && idx < tokenDefsLength,
        )

        entry.tokenTrigrams = validTokens.map(([idx, start, end]) => ({
          ...data.tokenDefs[idx],
          start,
          end,
        }))

        // Delete compact form - no longer needed after expansion
        delete entry.tokens
      }
    }
    return data.entries
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
      totalShards: this.manifest ? Object.keys(this.manifest.shards).length : 0,
    }
  }
}
