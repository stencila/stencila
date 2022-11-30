import { NodeSpec } from 'prosemirror-model'

/**
 * Generate a `NodeSpec` to represent a Stencila `List`
 *
 * See https://github.com/ProseMirror/prosemirror-schema-list/blob/1073b7b88ade52ef7cf3513d1ef29426b7b82c26/src/schema-list.ts
 * for slightly different node specs for lists.
 */
export function list(): NodeSpec {
  return {
    group: 'BlockContent',
    content: 'ListItem+',
    contentProp: 'items',
    attrs: {
      order: { default: 'Unordered' },
    },
    parseDOM: [
      { tag: 'ul', attrs: { order: 'Unordered' } },
      { tag: 'ol', attrs: { order: 'Ascending' } },
    ],
    toDOM(node) {
      return [node.attrs.order === 'Unordered' ? 'ul' : 'ol', 0]
    },
  }
}

/**
 * Generate a `NodeSpec` to represent a Stencila `ListItem`
 *
 * See https://github.com/ProseMirror/prosemirror-schema-list/blob/1073b7b88ade52ef7cf3513d1ef29426b7b82c26/src/schema-list.ts#L49
 * for why the `content` is defined as it is: to be able to use the commands in `prosemirror-schema-list`
 * package
 */
export function listItem(): NodeSpec {
  return {
    content: 'Paragraph BlockContent*',
    parseDOM: [{ tag: 'li' }],
    toDOM() {
      return ['li', 0]
    },
  }
}
