/**
 * Shared types for site action components
 *
 * This module defines the event types and interfaces used for
 * self-registration and communication between site actions and
 * the site-actions container.
 */

// Event type constants
export const SITE_ACTION_REGISTER = 'site-action-register'
export const SITE_ACTION_BADGE_UPDATE = 'site-action-badge-update'
export const SITE_ACTION_UNREGISTER = 'site-action-unregister'
export const SITE_ACTION_REQUEST_REGISTER = 'site-action-request-register'

/**
 * Registration detail dispatched by child actions
 */
export interface ActionRegistration {
  id: string
  icon: string
  label: string
  order: number
  openPanel: () => void // Must be arrow function to preserve binding
}

/**
 * Badge update detail dispatched when badge count changes
 */
export interface BadgeUpdateDetail {
  id: string
  count: number
}

/**
 * User info from auth status
 */
export interface AuthUser {
  id: string
  name: string
  avatar: string
}

/**
 * GitHub connection info from auth status
 */
export interface AuthGitHub {
  connected: boolean
  username: string
  canPush: boolean
  source: 'clerk' | 'oauth'
}

/**
 * Repository info from auth status
 */
export interface AuthRepo {
  isPrivate: boolean
  appInstalled: boolean
}

/**
 * Authorship info from auth status
 */
export interface AuthAuthorship {
  canAuthorAsSelf: boolean
  willBeBotAuthored: boolean
  reason?: string
}

/**
 * Review action config from auth status
 *
 * - `enabled`: Whether the review action is enabled for this site
 * - `allowed`: Whether the current user is allowed to submit reviews
 */
export interface ReviewConfig {
  enabled: boolean
  allowed: boolean
  allowPublic: boolean
  allowAnonymous: boolean
  types?: string[]
  minSelection?: number
  maxSelection?: number
}

/**
 * Upload action config from auth status
 *
 * - `enabled`: Whether the upload action is enabled for this site
 * - `allowed`: Whether the current user is allowed to upload files
 *
 * Note: public/anon visibility settings and other options (targetPath, userPath,
 * allowOverwrite, requireMessage) are passed as component attributes,
 * not in the auth response. The auth endpoint returns user-specific capabilities.
 */
export interface UploadConfig {
  enabled: boolean
  allowed: boolean
  allowedDirectories: string[] | null
  allowedExtensions: string[] | null
  maxFileSize: number
}

/**
 * Remote action config from auth status
 *
 * - `enabled`: Whether the remote action is enabled for this site
 * - `allowed`: Whether the current user is allowed to add remote documents
 *
 * Formats are returned without dots (e.g., "smd", "md", "html")
 *
 * Note: public/anon visibility settings are passed as component attributes,
 * not in the auth response. The auth endpoint returns user-specific capabilities.
 */
export interface RemoteConfig {
  enabled: boolean
  allowed: boolean
  targetPath: string
  userPath: boolean
  defaultFormat: string
  allowedFormats?: string[]
  defaultSyncDirection?: string
  requireMessage?: boolean
}

/**
 * Unified auth status response from /__stencila/auth/status
 *
 * This single endpoint serves all site actions (review, upload, remote).
 * Each action reads its specific config section.
 */
export interface SiteAuthStatusResponse {
  hasSiteAccess: boolean
  user?: AuthUser
  github?: AuthGitHub
  repo?: AuthRepo
  authorship?: AuthAuthorship
  reviewConfig?: ReviewConfig
  uploadConfig?: UploadConfig
  remoteConfig?: RemoteConfig
}

/**
 * @deprecated Use SiteAuthStatusResponse instead
 * Base auth response - kept for backwards compatibility during migration
 */
export interface BaseAuthStatusResponse {
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
 * Base footer states - actions can extend with additional states
 *
 * For 'success' state:
 * - prNumber/prUrl: Shown when PR was created immediately (upload, review)
 * - Without prNumber/prUrl: For async PR creation (remote) where PR will be created later
 */
export type BaseFooterState =
  | { type: 'loading' }
  | { type: 'submitting' }
  | { type: 'success'; prNumber?: number; prUrl?: string }
  | { type: 'error'; message: string }
  | { type: 'blocked'; reason: string }
  | { type: 'needSiteAccess'; signInUrl: string }
  | { type: 'needStencilaSignIn'; signInUrl: string }
  | { type: 'needGitHubConnect' }
  | { type: 'canSubmit'; authorDescription: string }

/**
 * Position type for FAB and panel placement
 */
export type ActionPosition =
  | 'bottom-right'
  | 'bottom-left'
  | 'top-right'
  | 'top-left'

/**
 * Result type for API submissions
 */
export type ApiSubmitResult<T> =
  | { ok: true; data: T }
  | { ok: false; error: string; statusCode?: number; errorData?: unknown }

/**
 * Custom event type declarations for TypeScript
 */
declare global {
  interface HTMLElementEventMap {
    [SITE_ACTION_REGISTER]: CustomEvent<ActionRegistration>
    [SITE_ACTION_BADGE_UPDATE]: CustomEvent<BadgeUpdateDetail>
    [SITE_ACTION_UNREGISTER]: CustomEvent<{ id: string }>
    [SITE_ACTION_REQUEST_REGISTER]: CustomEvent<void>
  }
}
