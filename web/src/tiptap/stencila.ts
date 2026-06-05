/**
 * Tiptap extensions for Stencila nodes that are not yet editable as native
 * ProseMirror content.
 *
 * Unsupported Stencila block and inline nodes are preserved as opaque atoms so
 * users can edit surrounding prose without dropping executable or structured
 * node payloads.
 */
import { Node } from '@tiptap/core'

/**
 * Shared attributes for opaque Stencila node placeholders.
 */
function opaqueAttributes(): Record<string, { default: unknown }> {
  return {
    nodeType: {
      default: null,
    },
    node: {
      default: null,
    },
  }
}

/**
 * Get a readable placeholder label from a serialized Stencila node type.
 */
function opaqueLabel(value: unknown, fallback: string): string {
  return typeof value === 'string' && value ? value : fallback
}

/**
 * Opaque block-level placeholder for unsupported Stencila block nodes.
 */
export const StencilaBlock = Node.create({
  name: 'stencilaBlock',

  group: 'block',
  atom: true,
  selectable: true,
  draggable: true,

  addAttributes() {
    return opaqueAttributes()
  },

  parseHTML() {
    return [{ tag: 'stencila-block-placeholder' }]
  },

  renderHTML({ node }) {
    const label = opaqueLabel(node.attrs.nodeType, 'Stencila block')

    return [
      'stencila-block-placeholder',
      {
        class: 'stencila-tiptap-opaque stencila-tiptap-opaque-block',
        contenteditable: 'false',
        'data-node-type': label,
      },
      label,
    ]
  },
})

/**
 * Opaque inline placeholder for unsupported Stencila inline nodes.
 */
export const StencilaInline = Node.create({
  name: 'stencilaInline',

  inline: true,
  group: 'inline',
  atom: true,
  selectable: true,

  addAttributes() {
    return opaqueAttributes()
  },

  parseHTML() {
    return [{ tag: 'stencila-inline-placeholder' }]
  },

  renderHTML({ node }) {
    const label = opaqueLabel(node.attrs.nodeType, 'Stencila inline')

    return [
      'stencila-inline-placeholder',
      {
        class: 'stencila-tiptap-opaque stencila-tiptap-opaque-inline',
        contenteditable: 'false',
        'data-node-type': label,
      },
      label,
    ]
  },
})
