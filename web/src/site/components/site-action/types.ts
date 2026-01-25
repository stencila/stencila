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
 * Base auth response - shared fields between all actions.
 * Each action extends this with action-specific config (reviewConfig, uploadConfig, etc.)
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
 */
export type BaseFooterState =
  | { type: 'loading' }
  | { type: 'submitting' }
  | { type: 'success'; prNumber: number; prUrl: string }
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
