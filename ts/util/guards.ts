import {
  BlockContent,
  blockContentTypes,
  Entity,
  InlineContent,
  inlineContentTypes,
  Node,
  TypeMap,
  TypeMapGeneric,
  Types,
} from '../types'

/**
 * Type guard to determine whether a node is a primitive type
 * (i.e. not an `Entity`).
 */
export const isPrimitive = (
  node?: Node
): node is null | boolean | number | string => {
  const type = typeof node
  if (node === null) return true
  if (type === 'boolean') return true
  if (type === 'number') return true
  if (type === 'string') return true
  if (Array.isArray(node)) return true
  if (type === 'object' && !Object.prototype.hasOwnProperty.call(node, 'type'))
    return true
  return false
}

/**
 * Type guard to determine whether a node is an `Entity`
 */
export const isEntity = (node?: Node): node is Entity => {
  if (node === null || node === undefined) return false
  return Object.prototype.hasOwnProperty.call(node, 'type')
}

/**
 * Returns a type guard to determine whether a node has a types
 * that is a member of the type map.
 *
 * e.g. isTypeOf('CreativeWork')(node)
 */
export const isTypeOf =
  <T extends Partial<TypeMap | TypeMapGeneric>>(typeMap: T) =>
  (node?: Node): boolean =>
    isEntity(node) && Object.keys(typeMap).includes(node.type)

/**
 * A type guard to determine whether a node is of a specific type.
 *
 * e.g. `isA('Paragraph', node)`
 */
export const isA = <K extends keyof Types>(
  type: K,
  node: Node | undefined
): node is Types[K] => isEntity(node) && node.type === type

/**
 * Returns a type guard to determine whether a node is of a specific type.
 *
 * e.g. `isType('Article')(node)`
 * e.g. `article.content.filter(isType('Paragraph'))`
 */
export const isType =
  <K extends keyof Types>(type: K) =>
  (node?: Node): node is Types[K] =>
    isA(type, node)

/**
 * Type guard to determine whether a node is `InlineContent`.
 *
 * e.g. `nodes.filter(isInlineContent)`
 */
export const isInlineContent = (node?: Node): node is InlineContent =>
  isPrimitive(node) || isTypeOf(inlineContentTypes)(node)

/**
 * Type guard to determine whether a node is `BlockContent`.
 *
 * e.g. `nodes.filter(isBlockContent)`
 */
export const isBlockContent = (node?: Node): node is BlockContent =>
  isTypeOf(blockContentTypes)(node)
