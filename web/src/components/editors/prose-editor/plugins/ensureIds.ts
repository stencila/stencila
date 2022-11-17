import { customAlphabet } from 'nanoid'
import { Node } from 'prosemirror-model'
import { Plugin } from 'prosemirror-state'

/**
 * Plugin to ensure that nodes have an `id`
 *
 * If the node has an `id` attribute that is `null` (on creation) or an empty string (on copy),
 * this function will generate a new id with a prefix based on the node type
 *
 * Based on https://discuss.prosemirror.net/t/how-i-can-attach-attribute-with-dynamic-value-when-new-paragraph-is-inserted/751/
 */
export function ensureIds() {
  return new Plugin({
    appendTransaction: (transactions, prevState, nextState) => {
      const tr = nextState.tr
      let modified = false
      if (transactions.some((transaction) => transaction.docChanged)) {
        nextState.doc.descendants((node, pos) => {
          const type = node.type.name
          if (type === 'CallArgument' || type === 'IfClause') {
            // These node types should not be assigned an id
            return
          }

          const id = (node as any).attrs?.id
          if (id === null || id === '') {
            const id = generateId(node)
            tr.setNodeAttribute(pos, 'id', id)
            modified = true
          }
        })
      }
      return modified ? tr : null
    },
  })
}

/**
 * Generate an id for a node
 *
 * This should generate the SUIDs ("Stencila Unique Identifiers") in
 * the same format as the Rust `suids` crate (two letter prefix, underscore,
 * twenty characters in [a-zA-Z0-9])
 */
export function generateId(node: Node) {
  const prefix = (() => {
    switch (node.type.name) {
      case 'Button':
        return 'bu'
      case 'Call':
        return 'ca'
      case 'CodeChunk':
        return 'cc'
      case 'CodeExpression':
        return 'ce'
      case 'CodeBlock':
        return 'cb'
      case 'CodeFragment':
        return 'cf'
      case 'Division':
        return 'di'
      case 'For':
        return 'fo'
      case 'Form':
        return 'fm'
      case 'If':
        return 'if'
      case 'Include':
        return 'in'
      case 'MathBlock':
        return 'mb'
      case 'MathFragment':
        return 'mf'
      case 'Parameter':
        return 'pa'
      case 'Span':
        return 'sp'
      default:
        return 'no'
    }
  })()
  return `${prefix}_${idGenerator()}`
}

const idGenerator = customAlphabet(
  '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz',
  20
)
