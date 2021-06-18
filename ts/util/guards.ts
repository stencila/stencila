import {
  BlockContent,
  Entity,
  InlineContent,
  Node,
  TypeMap,
  Types,
  unions,
  Unions,
} from '../types'

export type TypeMapGeneric<
  T extends { type: string } & Record<string, unknown> = { type: string }
> = { [key in T['type']]: key }

type ExtractGeneric<Type> = Type extends TypeMap<infer X>
  ? X
  : Type extends TypeMapGeneric<infer Y>
  ? Y
  : never

/**
 * Type guard to determine whether a node belongs to a type map
 *
 * @template {TypeMap} T
 * @param {T} typeMap
 * @param {Node} node A Stencila schema node object
 */
export const isInTypeMap =
  <T extends Partial<TypeMap | TypeMapGeneric>>(typeMap: T) =>
  (node?: Node): node is ExtractGeneric<T> =>
    isEntity(node) ? Object.keys(typeMap).includes(node.type) : false

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
 * A type guard to determine whether a node is a member of a union type.
 *
 * e.g. `isIn('MediaObjectTypes', node)`
 */
export const isIn = <K extends keyof Unions>(
  type: K,
  node: Node | undefined
): node is Unions[K] => isEntity(node) && node.type in unions[type]

/**
 * Returns a type guard to determine whether a node is a member of a union type.
 *
 * e.g. `isMember('CreativeWorkTypes')(node)`
 */
export const isMember =
  <K extends keyof Unions>(type: K) =>
  (node?: Node): node is Unions[K] =>
    isIn(type, node)

/**
 * Type guard to determine whether a node is `InlineContent`.
 *
 * e.g. `nodes.filter(isInlineContent)`
 */
export const isInlineContent = (node?: Node): node is InlineContent =>
  isPrimitive(node) || isIn('InlineContent', node)

/**
 * Type guard to determine whether a node is `BlockContent`.
 *
 * e.g. `nodes.filter(isBlockContent)`
 */
export const isBlockContent = (node?: Node): node is BlockContent =>
  isIn('BlockContent', node)
