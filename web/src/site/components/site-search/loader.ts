/**
 * Search index loader
 *
 * Manages loading and caching of search index shards.
 * Shards are organized by access level at `_search/{level}/shards/{prefix}.json`.
 */

import type { AccessLevel } from '../site-action/types'

import type {
  SearchEntry,
  SearchManifest,
  SearchRootManifest,
  ShardData,
} from './types'

/**
 * Manages loading and caching of search index shards
 *
 * The search index is sharded by access level, with each level containing
 * only entries at that level. Call `setAccessibleLevels()` to configure
 * which levels to load from based on user authentication.
 */
export class SearchIndexLoader {
  /** Root manifest listing available access levels */
  private rootManifest: SearchRootManifest | null = null
  private rootManifestLoading: Promise<SearchRootManifest> | null = null

  /** Per-level manifests, keyed by level name */
  private levelManifests: Map<string, SearchManifest> = new Map()
  private levelManifestsLoading: Map<string, Promise<SearchManifest>> =
    new Map()

  /** Shard cache, keyed by "{level}/{prefix}" */
  private shardCache: Map<string, SearchEntry[]> = new Map()
  private loadingShards: Map<string, Promise<SearchEntry[]>> = new Map()

  /** Access levels the user can access (defaults to public only) */
  private accessibleLevels: AccessLevel[] = ['public']

  private basePath: string

  constructor(basePath: string = '/_search') {
    this.basePath = basePath
  }

  /**
   * Set the access levels the user can access
   *
   * Call this with the user's accessible levels from auth status.
   * Example: For a team member, pass ['public', 'subscriber', 'password', 'team']
   */
  setAccessibleLevels(levels: AccessLevel[]): void {
    this.accessibleLevels = levels
  }

  /**
   * Get the currently configured accessible levels
   */
  getAccessibleLevels(): AccessLevel[] {
    return this.accessibleLevels
  }

  /**
   * Load the root manifest (call once at initialization)
   *
   * Safe to call concurrently - only one fetch will be made.
   */
  async loadRootManifest(): Promise<SearchRootManifest> {
    // Return cached manifest
    if (this.rootManifest) {
      return this.rootManifest
    }

    // Return in-flight promise if already loading
    if (this.rootManifestLoading) {
      return this.rootManifestLoading
    }

    // Start loading and cache the promise
    this.rootManifestLoading = this.fetchRootManifest()

    try {
      const manifest = await this.rootManifestLoading
      return manifest
    } finally {
      // Clear loading state (but keep manifest cached)
      this.rootManifestLoading = null
    }
  }

  /**
   * Fetch the root manifest
   */
  private async fetchRootManifest(): Promise<SearchRootManifest> {
    const response = await fetch(`${this.basePath}/manifest.json`)
    if (!response.ok) {
      throw new Error(`Failed to load search manifest: ${response.status}`)
    }

    const data = await response.json()

    if (!('levels' in data) || typeof data.levels !== 'object') {
      throw new Error('Invalid search manifest format: missing "levels"')
    }

    this.rootManifest = data as SearchRootManifest
    return this.rootManifest
  }

  /**
   * Load a per-level manifest
   */
  private async loadLevelManifest(
    level: string,
  ): Promise<SearchManifest | null> {
    // Check cache
    if (this.levelManifests.has(level)) {
      return this.levelManifests.get(level)!
    }

    // Check if already loading
    if (this.levelManifestsLoading.has(level)) {
      return this.levelManifestsLoading.get(level)!
    }

    // Check if level exists in root manifest
    if (this.rootManifest && !this.rootManifest.levels[level]) {
      return null
    }

    // Start loading
    const loadPromise = this.fetchLevelManifest(level)
    this.levelManifestsLoading.set(level, loadPromise)

    try {
      const manifest = await loadPromise
      this.levelManifests.set(level, manifest)
      return manifest
    } finally {
      this.levelManifestsLoading.delete(level)
    }
  }

  /**
   * Fetch a per-level manifest
   */
  private async fetchLevelManifest(level: string): Promise<SearchManifest> {
    const response = await fetch(`${this.basePath}/${level}/manifest.json`)
    if (!response.ok) {
      throw new Error(
        `Failed to load search manifest for ${level}: ${response.status}`,
      )
    }
    return response.json()
  }

  /**
   * Check if root manifest is loaded
   */
  isLoaded(): boolean {
    return this.rootManifest !== null
  }

  /**
   * Get total entry count across all accessible levels
   */
  getTotalEntries(): number {
    if (!this.rootManifest) {
      return 0
    }

    let total = 0
    for (const level of this.accessibleLevels) {
      const levelInfo = this.rootManifest.levels[level]
      if (levelInfo) {
        total += levelInfo.entryCount
      }
    }
    return total
  }

  /**
   * Load entries for a specific token prefix from all accessible levels
   */
  async loadShard(prefix: string): Promise<SearchEntry[]> {
    // Ensure root manifest is loaded
    if (!this.rootManifest) {
      await this.loadRootManifest()
    }

    // Load manifests and shards from all accessible levels in parallel
    const loadPromises: Promise<SearchEntry[]>[] = []

    for (const level of this.accessibleLevels) {
      // Skip if this level doesn't exist in the index
      if (!this.rootManifest!.levels[level]) {
        continue
      }

      loadPromises.push(this.loadShardFromLevel(level, prefix))
    }

    const results = await Promise.all(loadPromises)
    return results.flat()
  }

  /**
   * Load a shard from a specific access level
   */
  private async loadShardFromLevel(
    level: string,
    prefix: string,
  ): Promise<SearchEntry[]> {
    const cacheKey = `${level}/${prefix}`

    // Check cache first
    if (this.shardCache.has(cacheKey)) {
      return this.shardCache.get(cacheKey)!
    }

    // Ensure level manifest is loaded
    const levelManifest = await this.loadLevelManifest(level)
    if (!levelManifest) {
      return []
    }

    // Check if shard exists for this prefix in this level
    if (!levelManifest.shards[prefix]) {
      return []
    }

    // Check if this shard is already loading
    if (this.loadingShards.has(cacheKey)) {
      return this.loadingShards.get(cacheKey)!
    }

    // Load the shard
    const loadPromise = this.fetchShardFromLevel(level, prefix)
    this.loadingShards.set(cacheKey, loadPromise)

    try {
      const entries = await loadPromise
      // Cache entries for this level/prefix
      this.shardCache.set(cacheKey, entries)
      return entries
    } finally {
      this.loadingShards.delete(cacheKey)
    }
  }

  /**
   * Fetch a shard from a specific access level
   */
  private async fetchShardFromLevel(
    level: string,
    prefix: string,
  ): Promise<SearchEntry[]> {
    const response = await fetch(
      `${this.basePath}/${level}/shards/${prefix}.json`,
    )
    if (!response.ok) {
      throw new Error(
        `Failed to load shard ${level}/${prefix}: ${response.status}`,
      )
    }
    const data: unknown = await response.json()
    const shardData = this.validateShardData(data, `${level}/${prefix}`)
    const entries = this.expandTokens(shardData)

    // Tag entries with their access level (skip for public to save memory)
    if (level !== 'public') {
      for (const entry of entries) {
        entry.accessLevel = level as AccessLevel
      }
    }

    return entries
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
        `Shard ${prefix}: expected ShardData object, got ${Array.isArray(data) ? 'array' : typeof data}`,
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
   * Clear all caches (useful for memory management or after access level change)
   */
  clearCache(): void {
    this.shardCache.clear()
    this.levelManifests.clear()
  }

  /**
   * Get cache statistics
   */
  getCacheStats(): {
    cachedShards: number;
    cachedLevels: number;
    totalShards: number;
  } {
    // Count total shards across accessible levels
    let totalShards = 0
    if (this.rootManifest) {
      for (const level of this.accessibleLevels) {
        const levelInfo = this.rootManifest.levels[level]
        if (levelInfo) {
          totalShards += levelInfo.shardCount
        }
      }
    }

    return {
      cachedShards: this.shardCache.size,
      cachedLevels: this.levelManifests.size,
      totalShards,
    }
  }
}
