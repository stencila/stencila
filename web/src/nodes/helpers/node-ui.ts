import type { NodeType } from '@stencila/types'

export type NodeTypeUI = {
  title?: string
  icon?: string
  colour?: string
  borderColour?: string
}

const DEFAULT_NODE_ICON = 'admonition'
const DEFAULT_COLOUR = '#FFE6CC' // orange
const DEFAULT_BORDER_COLOUR = '#FF8000' // orange

// prettier-ignore
const nodeTypeUIMap: Partial<Record<NodeType, NodeTypeUI>> = {
  Admonition:       { icon: 'admonition' },
  ForBlock:         { icon: 'for-block' },
  Heading:          { icon: 'text',           colour: '#F5F3FC', borderColour: '#DED6F5' },
  IfBlock:          { icon: 'if-block' },
  InstructionBlock: { icon: 'instruction-block' },
  List:             { icon: 'list' },
  Paragraph:        { icon: 'paragraph',      colour: '#F5FFF5', borderColour: '#D9F2D9' },
  Table:            { icon: 'table',          colour: '#F5F3FC', borderColour: '#DED6F5' },
}

export const nodeUi = (nodeType: NodeType): Required<NodeTypeUI> => {
  const ui = nodeTypeUIMap[nodeType]
  return {
    title: ui?.title ?? nodeType.replace(/([A-Z])/g, ' $1').trim(),
    icon: ui?.icon ?? DEFAULT_NODE_ICON,
    colour: ui?.colour ?? DEFAULT_COLOUR,
    borderColour: ui?.borderColour ?? DEFAULT_BORDER_COLOUR,
  }
}

export const nodeTitle = (nodeType: NodeType) =>
  nodeTypeUIMap[nodeType]?.title ?? nodeType.replace(/([A-Z])/g, ' $1').trim()

export const nodeIcon = (nodeType: NodeType) =>
  nodeTypeUIMap[nodeType]?.icon ?? DEFAULT_NODE_ICON

export const nodeColour = (nodeType: NodeType) =>
  nodeTypeUIMap[nodeType]?.colour ?? DEFAULT_COLOUR

export const nodeBorderColour = (nodeType: NodeType) =>
  nodeTypeUIMap[nodeType]?.borderColour ?? DEFAULT_BORDER_COLOUR
