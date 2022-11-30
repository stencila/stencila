import { NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `Heading`.
 *
 * Note that, consistent with treatment elsewhere, `h2` => level 3 etc.
 * This is because there should only be one `h1` (for the title) and when encoding to
 * HTML we add one to the depth e.g. `depth: 1` => `h2`
 */
export function heading(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'InlineContent*',
    marks: '_',
    attrs: { depth: { default: 1 } },
    parseDOM: [
      { tag: 'h1', attrs: { depth: 1 } },
      { tag: 'h2', attrs: { depth: 1 } },
      { tag: 'h3', attrs: { depth: 2 } },
      { tag: 'h4', attrs: { depth: 3 } },
      { tag: 'h5', attrs: { depth: 4 } },
      { tag: 'h6', attrs: { depth: 5 } },
    ],
    toDOM(node) {
      return [`h${(node.attrs.depth as number) + 1}`, 0]
    },
  }
}
