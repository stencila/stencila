import { NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `QuoteBlock`
 */
export function quoteBlock(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'BlockContent*',
    parseDOM: [{ tag: 'blockquote' }],
    toDOM(node) {
      return ['blockquote', 0]
    },
  }
}
