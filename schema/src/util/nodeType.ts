import { Types, Node } from '../types'
import { isEntity } from './guards'

/**
 * Get the type of a node
 *
 * @param {Node} node The schema node to get the type for
 */
export const nodeType = (node: Node): keyof Types => {
  if (node === null) return 'Null'
  if (typeof node === 'boolean') return 'Boolean'
  if (typeof node === 'number') return 'Number'
  if (typeof node === 'string') return 'String'
  if (Array.isArray(node)) return 'Array'
  if (isEntity(node)) return node.type
  return 'Object'
}
