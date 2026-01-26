/**
 * Shared types for site upload components
 */

/**
 * A file pending upload
 */
export interface PendingFile {
  /** Unique ID for this pending file */
  id: string
  /** Original filename */
  filename: string
  /** Target path in the repository */
  targetPath: string
  /** File size in bytes */
  size: number
  /** MIME type */
  mimeType: string
  /** Whether this overwrites an existing file */
  isOverwrite: boolean
  /** Base64-encoded file content */
  content: string
}

/**
 * A file in the repository (for update mode)
 */
export interface RepoFile {
  path: string
  size: number
  lastModified: string
}

// Auth types (SiteAuthStatusResponse, UploadConfig) are defined in
// site-action/types.ts and shared across all site actions.

/**
 * Upload submission response from /__stencila/uploads
 */
export interface UploadResponse {
  success: boolean
  prNumber: number
  prUrl: string
  branchName: string
  authoredBy: 'user' | 'bot'
  authorUsername?: string
  filesAdded: number
  filesUpdated: number
}

/**
 * Error response from API
 */
export interface ApiError {
  error: string
  message?: string
  retryAfter?: number
}

/**
 * Footer state for the upload panel
 */
export type FooterState =
  | { type: 'loading' }
  | { type: 'uploading'; progress: number }
  | { type: 'success'; prNumber: number; prUrl: string }
  | { type: 'error'; message: string }
  | { type: 'blocked'; reason: string }
  | { type: 'needSiteAccess'; signInUrl: string }
  | { type: 'needStencilaSignIn'; signInUrl: string }
  | { type: 'needGitHubConnect' }
  | { type: 'canSubmit'; authorDescription: string; fileCount: number }
