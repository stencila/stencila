import { Node } from '../types'
import { isEntity } from './guards'
export * from './guards'
export * from './type-map'
export * from './type-maps'

/**
 * Get the type of a node
 * @param {Node} node The schema node to get the type for
 */
export const nodeType = (node: Node): string => {
  if (node === null) return 'null'
  if (typeof node === 'boolean') return 'boolean'
  if (typeof node === 'number') return 'number'
  if (typeof node === 'string') return 'string'
  if (Array.isArray(node)) return 'array'
  if (isEntity(node)) return node.type
  return typeof node
}
