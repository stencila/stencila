import type { NodeType } from '@stencila/types'

type NodeUISpecs = {
  icon?: string
  colour?: string
  borderColour?: string
}

const DEFAULT_NODE_ICON = 'admonition'
const DEFAULT_COLOUR = '#FF8000' // orange - for proof-of-concept only
const DEFAULT_BORDER_COLOUR = '#FFE6CC' // orange - for proof-of-concept only

const nodeTypeUIMap: { [k: NodeType]: NodeUISpecs } = {
  Heading: {
    icon: 'text',
    colour: '#F5F3FC', // purple-200
    borderColour: '#DED6F5', // purple-200
  },
  Table: {
    icon: 'table',
    colour: '#F5F3FC', // purple-200
    borderColour: '#DED6F5', // purple-200
  },
  Paragraph: {
    icon: 'paragraph',
    colour: '#F5FFF5', // green-000
    borderColour: '#D9F2D9', // green-200
  },
  List: { icon: 'list' },
  Admonition: {
    icon: 'admonition',
  },
  IfBlock: { icon: 'if-block' },
  ForBlock: { icon: 'for-block' },
  InstructionBlock: { icon: 'instruct-block' },
}

const getNodeIcon = (node: NodeType) => {
  return nodeTypeUIMap[node]?.icon ?? DEFAULT_NODE_ICON
}

const getNodeColour = (node: NodeType) => {
  return nodeTypeUIMap[node]?.colour ?? DEFAULT_COLOUR
}

const getNodeBorderColour = (node: NodeType) => {
  return nodeTypeUIMap[node]?.borderColour ?? DEFAULT_BORDER_COLOUR
}

export { getNodeIcon, getNodeColour, getNodeBorderColour }
export type { NodeUISpecs }
