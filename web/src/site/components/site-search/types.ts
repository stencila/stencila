/**
 * Type definitions for search index
 *
 * These types mirror the Rust structures in `rust/site/src/search/`
 */

/**
 * A single indexed entry in the search index
 */
export interface SearchEntry {
  /** The node ID (e.g., "hed_ABC123") */
  nodeId: string
  /** The type of node (e.g., "Heading", "Paragraph", "Datatable") */
  nodeType: string
  /** The route where this node appears (e.g., "/docs/guide/") */
  route: string
  /** The indexed text content */
  text: string
  /** Structural weight for ranking (higher = more important) */
  weight: number
  /** Depth in document (0=root article, 1=top-level section, etc.) */
  depth: number
  /** For datatables: additional metadata */
  metadata?: DatatableMetadata
}

/**
 * Additional metadata for datatable nodes
 */
export interface DatatableMetadata {
  /** Column names */
  columns: string[]
  /** Description if available */
  description?: string
  /** Row count hint */
  rowCount?: number
}

/**
 * Manifest file describing the search index
 */
export interface SearchManifest {
  /** Schema version for forward compatibility */
  version: number
  /** Total number of indexed entries across all shards */
  totalEntries: number
  /** Total number of indexed documents/routes */
  totalRoutes: number
  /** Information about each shard */
  shards: ShardInfo[]
}

/**
 * Information about a single shard file
 *
 * The token prefix can be derived from the filename (e.g., "shards/ab.json" → "ab").
 */
export interface ShardInfo {
  /** Shard filename (e.g., "shards/ab.json") */
  file: string
  /** Number of entries in this shard */
  entryCount: number
  /** File size in bytes (after any compression) */
  sizeBytes?: number
}

/**
 * A search result with score and highlights
 */
export interface SearchResult {
  /** The matched entry */
  entry: SearchEntry
  /** Relevance score (higher = better match) */
  score: number
  /** Highlighted text ranges */
  highlights: TextHighlight[]
}

/**
 * A recent search selection stored in localStorage
 */
export interface RecentSearch {
  /** The node ID (e.g., "hed_ABC123") */
  nodeId: string
  /** The type of node (e.g., "Heading", "Paragraph") */
  nodeType: string
  /** The route where this node appears */
  route: string
  /** The text content to display */
  text: string
}

/**
 * A highlighted text range within a search result
 */
export interface TextHighlight {
  /** Start character index */
  start: number
  /** End character index */
  end: number
}

/**
 * Options for search queries
 */
export interface SearchOptions {
  /** Maximum number of results to return */
  limit?: number
  /** Number of results to skip (for pagination) */
  offset?: number
  /** Filter by node types */
  nodeTypes?: string[]
  /** Filter by route prefixes */
  routes?: string[]
}
