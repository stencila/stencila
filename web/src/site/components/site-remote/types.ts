/**
 * Shared types for site remote components
 *
 * Note: Auth types (SiteAuthStatusResponse, RemoteConfig) are defined in
 * site-action/types.ts and shared across all site actions.
 */

/**
 * Watch sync direction options
 */
export type WatchDirection = 'from-remote' | 'bi' | 'to-remote'

/**
 * Watch sync mode options (includes 'none' for no sync)
 */
export type WatchMode = WatchDirection | 'none'

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
  /** Whether to create watch */
  watch: boolean
  /** Sync direction (if sync enabled) */
  watchDirection: WatchDirection
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
