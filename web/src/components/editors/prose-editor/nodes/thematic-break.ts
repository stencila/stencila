import { NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `ThematicBreak`
 */
export function thematicBreak(): NodeSpec {
  return {
    group: 'BlockContent',
    parseDOM: [{ tag: 'hr' }],
    toDOM: () => ['hr'],
  }
}
