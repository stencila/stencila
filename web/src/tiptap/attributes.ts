/**
 * Shared attribute helpers for Stencila Tiptap extensions.
 */
import type { Attributes } from '@tiptap/core'

/**
 * Build attributes that round-trip through the editor without becoming DOM
 * attributes. The Rust codec remains responsible for their wire shape.
 */
export function passthroughAttrs(...names: string[]): Attributes {
  const attributes: Attributes = {}
  for (const name of names) {
    attributes[name] = { default: null, rendered: false }
  }
  return attributes
}
