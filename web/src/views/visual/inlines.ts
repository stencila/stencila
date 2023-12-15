import { type NodeSpec, NodeViewConstructor, attrsParseToDOM } from './prelude'

/**
 * A ProseMirror `NodeSpec` for a Stencila `Parameter` node
 */
const Parameter: NodeSpec = {
  group: 'Inline',
  inline: true,
  atom: true,
  ...attrsParseToDOM('stencila-parameter', 'id'),
}

/**
 * A ProseMirror `NodeSpec` for a Stencila `Text` node
 */
const Text: NodeSpec = {
  group: 'Inline',
  inline: true,
  content: 'text*',
  marks: '',
  ...attrsParseToDOM('span', 'id'),
}

// Export specs and views

export const specs: Record<string, NodeSpec> = {
  Parameter,
  Text,
  // Every schema needs to have a "text" type with no attributes
  text: { group: 'Inline' },
}

export const views: Record<string, NodeViewConstructor> = {}
