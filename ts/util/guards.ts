import {
  Article,
  blockContentTypes,
  codeTypes,
  creativeWorkTypes,
  Entity,
  InlineContent,
  inlineContentTypes,
  ListItem,
  Node,
  Paragraph,
  TypeMap,
  TypeMapGeneric,
  Types,
} from '../types'

type ExtractGeneric<Type> = Type extends TypeMap<infer X>
  ? X
  : Type extends TypeMapGeneric<infer Y>
  ? Y
  : never

/**
 * Returns a function which returns true is the type is a member
 * of the type map.
 *
 * @param {TypeMap} typeMap An object containing schema type values
 */
export const typeIs = <T extends Partial<TypeMap | TypeMapGeneric>>(
  typeMap: T
) => (type: string): boolean => Object.keys(typeMap).includes(type)

/**
 * Returns a type guard to determine whether a node belongs to a set
 * of types.
 * Returns a boolean value and narrows the TypeScript inferred type to
 * the type.
 *
 * @template {TypeMap} T
 * @param {T} typeMap
 * @param {Node} node A Stencila schema node object
 */
export const nodeIs = <T extends Partial<TypeMap | TypeMapGeneric>>(
  typeMap: T
) => (node?: Node): node is ExtractGeneric<T> => {
  return isEntity(node) ? typeIs(typeMap)(node.type) : false
}

/**
 * Returns a type guard to determine whether a node is of a particular type.
 * Returns a boolean value and narrows the TypeScript inferred type to
 * the type.
 *
 * @param type The type to test for
 */
// eslint-disable-next-line @typescript-eslint/explicit-function-return-type, @typescript-eslint/explicit-module-boundary-types
export const is = <Ts extends Entity>(type: keyof TypeMap<Ts>) => {
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  const typeMap: TypeMap<Ts> = {
    [type]: type,
  }
  return nodeIs(typeMap)
}

/**
 * A type guard to determine whether a node is of a specific type.
 * Returns a boolean value and narrows the TypeScript inferred type to
 * the type.
 *
 * e.g. `isA('Paragraph', node)`
 *
 * @param type The name of the type to test for
 * @param node The node being tested
 */
export const isA = <K extends keyof Types>(
  type: K,
  node: Node | undefined
): node is Types[K] => {
  return isEntity(node) && node.type === type
}

export const isInstanceOf = <
  E = never,
  TM = TypeMap<E extends Entity ? E : never>
>(
  typeMap: E extends never ? never : TM,
  node?: Node
): node is E => {
  return isEntity(node) && Object.keys(typeMap).includes(node.type)
}

/**
 * Returns a type guard to determine whether a node is of a specific type.
 * Returns a boolean value and narrows the TypeScript inferred type to
 * the type.
 *
 * e.g. `article.content.filter(isType('Paragraph'))`
 *
 * @param type The type to test for
 */
export const isType = <K extends keyof Types>(type: K) => (
  node?: Node
): node is Types[K] => {
  return node !== undefined && isA(type, node)
}

/**
 * Type guard to determine whether a node is a primitive type.
 * Returns a boolean value and narrows the TypeScript inferred type.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is null | boolean | number | string)} Returns true if node is one of `null`, `boolean`, `string`, or `number`
 */
export const isPrimitive = (
  node?: Node
): node is null | boolean | number | string => {
  const type = typeof node
  if (node === null) return true
  if (type === 'boolean') return true
  if (type === 'number') return true
  if (type === 'string') return true
  return false
}

/**
 * Type guard to determine whether a node is an `Entity`
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is Entity)} Returns true if node is an `Entity` or derived type
 */
export const isEntity = (node?: Node): node is Entity => {
  if (node === null || node === undefined) return false
  return Object.prototype.hasOwnProperty.call(node, 'type')
}

/**
 * Type guard to determine whether a node is both `InlineContent` and
 * and an `Entity`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is InlineContent)}
 */
export const isInlineEntity = (node?: Node): node is InlineContent => {
  return typeof node === 'object' && isEntity(node)
    ? typeIs(inlineContentTypes)(node.type)
    : false
}
/**
 * Type guard to determine whether a node is `InlineContent`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is InlineContent)}
 */
export const isInlineContent = (node?: Node): node is InlineContent => {
  return isPrimitive(node) || isInlineEntity(node)
}

/**
 * Type guard to determine whether a node is `BlockContent`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is BlockContent)}
 */
export const isBlockContent = nodeIs(blockContentTypes)

/**
 * Type guard to determine whether a node is a `CreativeWork`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is CreativeWork)}
 */
export const isCreativeWork = nodeIs(creativeWorkTypes)

/**
 * Type guard to determine whether a node is a `Code`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is Code)}
 */
export const isCode = nodeIs(codeTypes)

/**
 * Type guard to determine whether a node is an `Article`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is Article)}
 */
export const isArticle = is<Article>('Article')

/**
 * Type guard to determine whether a node is an `Paragraph`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is Paragraph)}
 */
export const isParagraph = is<Paragraph>('Paragraph')

/**
 * Type guard to determine whether a node is an `ListItem`.
 *
 * @param {Node} node The node to get the type for
 * @returns {(node is ListItem)}
 */
export const isListItem = is<ListItem>('ListItem')
