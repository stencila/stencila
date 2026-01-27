/**
 * A file entry in the files index
 */
export interface FileEntry {
  /** Relative path from site root (e.g., "docs/guide/data.csv") */
  path: string
  /** File size in bytes */
  size: number
  /** File extension (without leading dot, lowercase) */
  extension: string
  /** Last modified timestamp (ISO 8601) */
  lastModified: string
}
