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

/**
 * Auth status response from /__stencila-upload/auth
 */
export interface UploadAuthStatusResponse {
  hasSiteAccess: boolean
  user?: {
    id: string
    name: string
    avatar: string
  }
  github?: {
    connected: boolean
    username: string
    canPush: boolean
    source: 'clerk' | 'oauth'
  }
  uploadConfig: {
    enabled: boolean
    allowPublic: boolean
    allowAnonymous: boolean
    allowedTypes: string[] | null
    maxSize: number
    targetPath: string
    userPath: boolean
    allowOverwrite: boolean
    requireMessage: boolean
  }
  repo?: {
    isPrivate: boolean
    appInstalled: boolean
  }
  authorship?: {
    canAuthorAsSelf: boolean
    willBeBotAuthored: boolean
    reason?: string
  }
}

/**
 * Upload submission response from /__stencila-upload/submit
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
