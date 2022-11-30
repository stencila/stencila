import { NodeSpec } from 'prosemirror-model'

export function paragraph(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'InlineContent*',
    parseDOM: [{ tag: 'p' }],
    toDOM: () => ['p', 0],
  }
}
