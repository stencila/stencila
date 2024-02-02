import { NodeType } from '../../types'

type NodeUISpecs = {
  icon?: string
  colour?: string
}

const DEFAULT_NODE_ICON = 'admonition'
const DEFAULT_COLOUR = '#FF8000' // orange - for proof-of-concept only

const nodeTypeUIMap: { [k: NodeType]: NodeUISpecs } = {
  Heading: {
    icon: 'text',
    colour: '#CDC1F0', // purple-200
  },
  Table: {
    icon: 'table',
    colour: '#CDC1F0', // purple-200
  },
  Paragraph: {
    icon: 'paragraph',
    colour: '#D9F2D9', // green-200
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

export { getNodeIcon, getNodeColour }
export type { NodeUISpecs }
