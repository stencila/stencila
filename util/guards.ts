import {
  Article,
  BlockContent,
  CreativeWork,
  InlineContent,
  ListItem,
  Node,
  Paragraph,
  Thing
} from '../types'
import { nodeType } from './index'

/**
 * TypeScript guard to determine whether a Node is a [Thing](../schema/Thing.schema.yaml)
 * Returns a boolean value and narrows the TypeScript inferred type.
 *
 * @param {Node} node The schema node to get the type for
 */
export const hasTypeProp = (node?: Node): node is Thing => {
  if (node === null || node === undefined) return false
  return Object.prototype.hasOwnProperty.call(node, 'type')
}

export type TypeMap<T extends Thing = Thing> = { [key in T['type']]: key }
export type TypeMapGeneric<
  T extends { type: string } & object = { type: string }
> = { [key in T['type']]: key }

type ExtractGeneric<Type> = Type extends TypeMap<infer X>
  ? X
  : Type extends TypeMapGeneric<infer Y>
  ? Y
  : never

/**
 * @param {TypeMap} typeMap An object containing schema type values
 * @param {string} nodeType string representation of the node type
 */
export const isOfType = <T extends Partial<TypeMap | TypeMapGeneric>>(
  typeMap: T
) => (node: string | Node): boolean =>
  Object.keys(typeMap).includes(
    typeof node === 'string' ? node : nodeType(node)
  )

/**
 * @template {TypeMap} T
 * @param {T} typeMap
 * @param {Node} node A Stencila schema node object
 */
export const nodeIsOfType = <T extends Partial<TypeMap | TypeMapGeneric>>(
  typeMap: T
) => (node?: Node): node is ExtractGeneric<T> => {
  return hasTypeProp(node) ? isOfType(typeMap)(node.type) : false
}

// eslint-disable-next-line
export const is = <Ts extends Thing>(type: keyof TypeMap<Ts>) => {
  // @ts-ignore
  const typeMap: TypeMap<Ts> = {
    [type]: type
  }
  return nodeIsOfType(typeMap)
}

export const blockContentTypes: TypeMap<BlockContent> = {
  CodeBlock: 'CodeBlock',
  CodeChunk: 'CodeChunk',
  Heading: 'Heading',
  List: 'List',
  ListItem: 'ListItem',
  Paragraph: 'Paragraph',
  QuoteBlock: 'QuoteBlock',
  Table: 'Table',
  ThematicBreak: 'ThematicBreak'
}

export const isBlockContent = nodeIsOfType(blockContentTypes)

type InlineNodesWithType = Exclude<
  InlineContent,
  string | null | boolean | number
>

export const inlineContentTypes: TypeMap<InlineNodesWithType> = {
  Code: 'Code',
  CodeBlock: 'CodeBlock',
  CodeExpr: 'CodeExpr',
  Delete: 'Delete',
  Emphasis: 'Emphasis',
  ImageObject: 'ImageObject',
  Link: 'Link',
  Quote: 'Quote',
  Strong: 'Strong',
  Subscript: 'Subscript',
  Superscript: 'Superscript'
}

/**
 *
 * @param {Node} node The schema node to get the type for
 * @returns {(node is null | boolean | number | string)} Returns true if node is one of `null`, `boolean`, `string`, or `number`
 */
export const isInlinePrimitive = (
  node: Node
): node is null | boolean | number | string => {
  const type = typeof node
  if (node === null) return true
  if (type === 'boolean') return true
  if (type === 'number') return true
  if (type === 'string') return true
  return false
}

export const isInlineNonPrimitive = (
  node: Node
): node is InlineNodesWithType => {
  return typeof node === 'object' && hasTypeProp(node)
    ? isOfType(inlineContentTypes)(node.type)
    : false
}

export const isInlineContent = (node: Node): node is InlineContent => {
  return isInlinePrimitive(node) || isInlineNonPrimitive(node)
}

export const creativeWorkTypes: TypeMap<CreativeWork> = {
  CreativeWork: 'CreativeWork',
  Article: 'Article',
  AudioObject: 'AudioObject',
  CodeChunk: 'CodeChunk',
  CodeExpr: 'CodeExpr',
  Collection: 'Collection',
  Datatable: 'Datatable',
  ImageObject: 'ImageObject',
  MediaObject: 'MediaObject',
  SoftwareApplication: 'SoftwareApplication',
  SoftwareSourceCode: 'SoftwareSourceCode',
  Table: 'Table',
  VideoObject: 'VideoObject'
}

export const isCreativeWork = nodeIsOfType(creativeWorkTypes)

export const isArticle = is<Article>('Article')
export const isParagraph = is<Paragraph>('Paragraph')
export const isListItem = is<ListItem>('ListItem')
