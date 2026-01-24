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
  nodeId: string;
  /** The type of node (e.g., "Heading", "Paragraph", "Datatable") */
  nodeType: string;
  /** The route where this node appears (e.g., "/docs/guide/") */
  route: string;
  /** Breadcrumb labels for the route (e.g., ["Home", "Docs", "Guide"]) */
  breadcrumbs: string[];
  /** The indexed text content */
  text: string;
  /** Structural weight for ranking (higher = more important) */
  weight: number;
  /** Depth in document (0=root article, 1=top-level section, etc.) */
  depth: number;
  /** For datatables: additional metadata */
  metadata?: DatatableMetadata;
  /** Compact token refs from JSON: [[defIndex, start, end], ...] */
  tokens?: Array<[number, number, number]>;
  /** Expanded token trigrams for fuzzy matching (populated at load time) */
  tokenTrigrams?: TokenTrigrams[];
  /** Cached tokenized text (internal, populated lazily on first search) */
  _cachedTokens?: string[];
}

/**
 * Token with pre-computed trigrams for fuzzy matching
 *
 * This is the expanded form used at runtime after loading from shard files.
 */
export interface TokenTrigrams {
  /** Normalized token (lowercased, diacritics folded) for trigram matching */
  token: string;
  /** Pre-computed character trigrams (3-grams) */
  trigrams: string[];
  /** Start position in original entry.text (UTF-16 code unit offset) */
  start: number;
  /** End position in original entry.text (UTF-16 code unit offset) */
  end: number;
}

/**
 * Token definition for fuzzy matching (shard-level, deduplicated)
 *
 * Contains the token string and its trigrams, without position information.
 * Stored once per unique token in a shard file.
 */
export interface TokenDef {
  /** Normalized token (lowercased, diacritics folded) */
  token: string;
  /** Pre-computed character trigrams (3-grams) */
  trigrams: string[];
}

/**
 * Shard file structure with deduplicated tokens
 *
 * Contains token definitions (deduplicated) and entries with compact token references.
 */
export interface ShardData {
  /** Deduplicated token definitions for fuzzy matching */
  tokenDefs: TokenDef[];
  /** Search entries with compact token references */
  entries: SearchEntry[];
}

/**
 * Additional metadata for datatable nodes
 */
export interface DatatableMetadata {
  /** Column names */
  columns: string[];
  /** Description if available */
  description?: string;
  /** Row count hint */
  rowCount?: number;
}

/**
 * Manifest file describing the search index
 */
export interface SearchManifest {
  /** Schema version for forward compatibility */
  version: number;
  /** Total number of indexed entries across all shards */
  totalEntries: number;
  /** Total number of indexed documents/routes */
  totalRoutes: number;
  /** Information about each shard, keyed by prefix (e.g., "ab", "co") */
  shards: Record<string, ShardInfo>;
}

/**
 * Information about a single shard file
 *
 * The shard file can be derived as `shards/{prefix}.json`.
 */
export interface ShardInfo {
  /** Number of entries in this shard */
  entryCount: number;
}

/**
 * A search result with score and highlights
 */
export interface SearchResult {
  /** The matched entry */
  entry: SearchEntry;
  /** Relevance score (higher = better match) */
  score: number;
  /** Highlighted text ranges */
  highlights: TextHighlight[];
}

/**
 * A recent search selection stored in localStorage
 */
export interface RecentSearch {
  /** The node ID (e.g., "hed_ABC123") */
  nodeId: string;
  /** The type of node (e.g., "Heading", "Paragraph") */
  nodeType: string;
  /** The route where this node appears */
  route: string;
  /** Breadcrumb labels for the route (e.g., ["Home", "Docs", "Guide"]) */
  breadcrumbs: string[];
  /** The text content to display */
  text: string;
  /** Depth in document (0=root/whole page, 1+=specific element) */
  depth: number;
}

/**
 * A highlighted text range within a search result
 */
export interface TextHighlight {
  /** Start character index */
  start: number;
  /** End character index */
  end: number;
}

/**
 * Options for search queries
 */
export interface SearchOptions {
  /** Maximum number of results to return */
  limit?: number;
  /** Number of results to skip (for pagination) */
  offset?: number;
  /** Filter by node types */
  nodeTypes?: string[];
  /** Filter by route prefixes */
  routes?: string[];
  /** Enable fuzzy matching (default: true if tokenTrigrams present) */
  enableFuzzy?: boolean;
  /** Minimum trigram similarity for fuzzy matches (0.0-1.0, default: 0.3) */
  fuzzyThreshold?: number;
}

/**
 * A single term in a parsed search query
 *
 * Represents either a quoted (required) term or an unquoted (optional/boost) term.
 */
export interface QueryTerm {
  /** The tokenized form of the term (e.g., ["getting", "started"]) */
  tokens: string[];
  /** Whether this term is required (true if quoted) */
  required: boolean;
  /** Whether tokens must appear adjacent (true for multi-word quoted phrases) */
  adjacentRequired: boolean;
}

/**
 * A parsed search query with required and optional terms
 *
 * Created by parsing a query string that may contain quoted phrases.
 * Example: `"cats" and dogs` produces:
 * - One required term: ["cats"]
 * - Two optional terms: ["and"], ["dogs"]
 */
export interface ParsedQuery {
  /** All parsed terms (both required and optional) */
  terms: QueryTerm[];
  /** Flat list of all tokens for shard loading */
  allTokens: string[];
}
