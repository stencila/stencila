/**
 * DOM, Range, and Selection utilities for site review components
 */

import type { ReviewItem } from './types'

/**
 * Check if CSS Custom Highlight API is supported
 */
export function supportsHighlightAPI(): boolean {
  return 'highlights' in CSS && typeof Highlight !== 'undefined'
}

// ============================================================================
// Text Position Utilities
// ============================================================================

/**
 * Find the text node and offset within it for a given element and character offset.
 * The character offset is relative to the element's total text content.
 */
export function findTextPosition(
  element: Element,
  charOffset: number
): { node: Text; offset: number } | null {
  const walker = document.createTreeWalker(element, NodeFilter.SHOW_TEXT)
  let currentOffset = 0
  let node: Text | null

  while ((node = walker.nextNode() as Text | null)) {
    const nodeLength = node.length
    if (currentOffset + nodeLength >= charOffset) {
      return { node, offset: charOffset - currentOffset }
    }
    currentOffset += nodeLength
  }

  // If offset exceeds content, return end of last text node
  if (node) {
    return { node, offset: node.length }
  }

  return null
}

/**
 * Find the closest ancestor element with an id attribute
 */
export function findNodeWithId(node: Node): Element | null {
  let current: Node | null = node
  while (current) {
    if (current instanceof Element && current.id) {
      return current
    }
    current = current.parentElement
  }
  return null
}

/**
 * Calculate character offset within a node's text content
 */
export function getCharOffset(nodeEl: Element, container: Node, offset: number): number {
  const preCaretRange = document.createRange()
  preCaretRange.selectNodeContents(nodeEl)
  preCaretRange.setEnd(container, offset)
  return preCaretRange.toString().length
}

// ============================================================================
// Range Utilities
// ============================================================================

/**
 * Create a Range for a review item's selection
 */
export function createRangeForItem(item: ReviewItem): Range | null {
  if (!item.start.nodeId || !item.end.nodeId) return null

  const startEl = document.getElementById(item.start.nodeId)
  const endEl = document.getElementById(item.end.nodeId)

  if (!startEl || !endEl) return null

  const startPos = findTextPosition(startEl, item.start.offset)
  const endPos = findTextPosition(endEl, item.end.offset)

  if (!startPos || !endPos) return null

  try {
    const range = document.createRange()
    range.setStart(startPos.node, startPos.offset)
    range.setEnd(endPos.node, endPos.offset)
    return range
  } catch (e) {
    console.warn('[SiteReview] Failed to create range:', e)
    return null
  }
}

/**
 * Get a collapsed Range at a screen position (cross-browser)
 * Chrome uses caretRangeFromPoint, Firefox uses caretPositionFromPoint
 */
export function caretRangeFromPoint(x: number, y: number): Range | null {
  // Chrome/Safari/Edge
  if (document.caretRangeFromPoint) {
    return document.caretRangeFromPoint(x, y)
  }

  // Firefox
  type CaretPosition = { offsetNode: Node; offset: number }
  type DocWithCaret = { caretPositionFromPoint?: (x: number, y: number) => CaretPosition | null }
  const caretPosition = (document as unknown as DocWithCaret).caretPositionFromPoint?.(x, y)
  if (caretPosition) {
    const { offsetNode, offset } = caretPosition

    // Validate offset to prevent "Index or size is negative or greater than allowed" error
    // For Text nodes, offset must be <= node.length
    // For Element nodes, offset must be <= childNodes.length
    const maxOffset =
      offsetNode.nodeType === Node.TEXT_NODE
        ? (offsetNode as Text).length
        : offsetNode.childNodes.length

    if (offset < 0 || offset > maxOffset) {
      return null
    }

    try {
      const range = document.createRange()
      range.setStart(offsetNode, offset)
      range.collapse(true)
      return range
    } catch {
      // If range creation still fails, return null gracefully
      return null
    }
  }

  return null
}

/**
 * Check if a range contains a point (represented as a collapsed range)
 */
export function rangeContainsPoint(range: Range, point: Range): boolean {
  return (
    range.compareBoundaryPoints(Range.START_TO_START, point) <= 0 &&
    range.compareBoundaryPoints(Range.END_TO_END, point) >= 0
  )
}

// ============================================================================
// DOM Traversal
// ============================================================================

/**
 * Find all elements between start and end nodeIds (inclusive).
 * Uses DOM tree order traversal to find all elements with IDs in the range.
 */
export function findElementsBetween(startNodeId: string, endNodeId: string): Element[] {
  if (!startNodeId || !endNodeId) return []

  const startEl = document.getElementById(startNodeId)
  const endEl = document.getElementById(endNodeId)

  if (!startEl || !endEl) return []

  // Same element - return just that one
  if (startNodeId === endNodeId) return [startEl]

  const result: Element[] = []
  let capturing = false

  // Get the main content area (look for common content containers)
  const contentSelector = 'main, [role="main"], article, [root]'
  const content = document.querySelector(contentSelector)
  if (!content) {
    // Fallback: just return start and end elements
    return [startEl, endEl]
  }

  // TreeWalker for efficient DOM traversal - only visit elements with IDs
  const walker = document.createTreeWalker(content, NodeFilter.SHOW_ELEMENT, {
    acceptNode: (node) => {
      const el = node as Element
      return el.id ? NodeFilter.FILTER_ACCEPT : NodeFilter.FILTER_SKIP
    },
  })

  let node: Node | null = walker.currentNode
  while (node) {
    const el = node as Element
    if (el.id === startNodeId) {
      capturing = true
    }

    if (capturing && el.id) {
      result.push(el)
    }

    if (el.id === endNodeId) {
      break
    }

    node = walker.nextNode()
  }

  // If we didn't capture anything due to tree structure, at least return start/end
  if (result.length === 0) {
    return [startEl, endEl].filter(Boolean)
  }

  return result
}
