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
