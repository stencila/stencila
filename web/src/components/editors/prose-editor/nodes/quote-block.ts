import { NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `QuoteBlock`
 */
export function quoteBlock(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'BlockContent+',
    // Necessary for copy/paste-ability of whole node, not just its content
    defining: true,
    parseDOM: [{ tag: 'blockquote' }],
    toDOM(node) {
      return ['blockquote', 0]
    },
  }
}
