/**
 * Shared types for site review components
 */

/**
 * Anchor point for selection (supports multi-block)
 */
export interface ReviewItemAnchor {
  nodeId: string
  offset: number
}

/**
 * Individual annotation item within a review
 */
export interface ReviewItem {
  type: 'comment' | 'suggestion'
  path: string
  url: string
  title: string
  start: ReviewItemAnchor
  end: ReviewItemAnchor
  selected: string
  content: string
}

/**
 * Source info from the root element (stencila-article)
 * repository and commit are site-wide; path varies per page
 */
export interface SourceInfo {
  repository: string
  commit: string
}

/**
 * Auth status response from /__stencila-review/auth
 */
export interface AuthStatusResponse {
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
  reviewConfig: {
    enabled: boolean
    allowPublic: boolean
    allowAnonymous: boolean
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
 * Review submission response from /__stencila-review/submit
 */
export interface ReviewResponse {
  success: boolean
  prNumber: number
  prUrl: string
  branchName: string
  authoredBy: 'user' | 'bot'
  authorUsername?: string
  usedFork: boolean
  forkFullName?: string
  counts: {
    comments: number
    suggestions: number
    fallbacks: number
  }
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
 * Footer state for the review panel
 * Evaluated in priority order - first matching state wins
 */
export type FooterState =
  | { type: 'loading' }
  | { type: 'submitting' }
  | { type: 'success'; prNumber: number; prUrl: string }
  | { type: 'error'; message: string }
  | { type: 'blocked'; reason: string }
  | { type: 'needSiteAccess'; signInUrl: string }
  | { type: 'needStencilaSignIn'; signInUrl: string }
  | { type: 'needGitHubConnect' }
  | { type: 'canSubmit'; authorDescription: string }
