/**
 * LRU page cache for client-side navigation
 *
 * Stores fetched page content for instant back/forward navigation
 * and repeated visits. Uses a Map-based LRU implementation with
 * configurable size limit.
 */

import type { CacheEntry } from './types'

/** Default maximum number of pages to cache */
const DEFAULT_MAX_SIZE = 10

/**
 * LRU (Least Recently Used) page cache
 *
 * Uses Map's insertion order to track recency - the oldest entries
 * are at the beginning and newest at the end. When capacity is
 * exceeded, the oldest entry is evicted.
 */
export class PageCache {
  /** Internal cache storage */
  private cache: Map<string, CacheEntry>

  /** Maximum number of entries to store */
  private maxSize: number

  constructor(maxSize: number = DEFAULT_MAX_SIZE) {
    this.cache = new Map()
    this.maxSize = maxSize
  }

  /**
   * Get a cached entry by URL
   *
   * If found, the entry is moved to the end (most recently used).
   */
  get(url: string): CacheEntry | undefined {
    const entry = this.cache.get(url)

    if (entry) {
      // Move to end by deleting and re-inserting
      this.cache.delete(url)
      this.cache.set(url, entry)
    }

    return entry
  }

  /**
   * Store a page in the cache
   *
   * If the cache is at capacity, evicts the least recently used entry.
   */
  set(url: string, entry: CacheEntry): void {
    // If already exists, delete first to update position
    if (this.cache.has(url)) {
      this.cache.delete(url)
    }

    // Evict oldest entry if at capacity
    if (this.cache.size >= this.maxSize) {
      const oldestKey = this.cache.keys().next().value
      if (oldestKey !== undefined) {
        this.cache.delete(oldestKey)
      }
    }

    this.cache.set(url, entry)
  }

  /**
   * Check if a URL is cached
   */
  has(url: string): boolean {
    return this.cache.has(url)
  }

  /**
   * Remove a specific entry from the cache
   */
  delete(url: string): boolean {
    return this.cache.delete(url)
  }

  /**
   * Clear all cached entries
   */
  clear(): void {
    this.cache.clear()
  }

  /**
   * Get the current number of cached entries
   */
  get size(): number {
    return this.cache.size
  }

  /**
   * Iterate over all cached URLs
   */
  keys(): IterableIterator<string> {
    return this.cache.keys()
  }

  /**
   * Iterate over all cached entries
   */
  entries(): IterableIterator<[string, CacheEntry]> {
    return this.cache.entries()
  }
}

/** Shared page cache instance */
let sharedCache: PageCache | null = null

/**
 * Get the shared page cache instance
 *
 * Creates the cache on first access with default settings.
 */
export function getPageCache(): PageCache {
  if (!sharedCache) {
    sharedCache = new PageCache()
  }
  return sharedCache
}

/**
 * Initialize the page cache with custom settings
 *
 * Should be called before any navigation occurs if custom
 * settings are needed.
 */
export function initPageCache(maxSize?: number): PageCache {
  sharedCache = new PageCache(maxSize)
  return sharedCache
}
