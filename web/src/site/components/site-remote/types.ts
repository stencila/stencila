/**
 * Shared types for site remote components
 *
 * Note: Auth types (SiteAuthStatusResponse, RemoteConfig) are defined in
 * site-action/types.ts and shared across all site actions.
 */

/**
 * Sync direction options
 */
export type SyncDirection = 'from-remote' | 'bi' | 'to-remote'

/**
 * Sync mode options (includes 'none' for no sync)
 */
export type SyncMode = SyncDirection | 'none'

/**
 * Remote service types
 */
export type RemoteService = 'gdoc' | 'm365'

/**
 * A pending remote document to add
 */
export interface PendingRemote {
  /** Remote document URL */
  url: string
  /** Document title from picker */
  title: string
  /** Service type */
  service: RemoteService
  /** Target path in the repository */
  targetPath: string
  /** Target format (without dot, e.g., "smd", "md", "html") */
  format: string
  /** Whether to enable sync */
  enableSync: boolean
  /** Sync direction (if sync enabled) */
  syncDirection: SyncDirection
}

/**
 * Remote submission response from POST /__stencila/remotes
 */
export interface RemoteResponse {
  success: boolean
  prNumber: number
  prUrl: string
  branchName: string
  authoredBy: 'user' | 'bot'
  authorUsername?: string
  watchId?: string
  syncDirection?: SyncDirection
}

/**
 * Picker message payload from stencila.cloud picker
 */
export interface PickerMessage {
  type: 'document-selected'
  url: string
  title: string
  service: RemoteService
}

/**
 * Error response from API
 */
export interface ApiError {
  error: string
  message?: string
  retryAfter?: number
}
