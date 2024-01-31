import { NodeType } from '../../types'

const DEFAULT_NODE_ICON = 'admonition'

const nodeTypeIconMap: { [k: NodeType]: string } = {
  Admonition: 'admonition',
  Paragraph: 'paragraph',
  IfBlock: 'if-block',
  ForBlock: 'for-block',
  List: 'list',
  InstructBlock: 'instruct-block',
  Heading: 'text',
  Table: 'table',
}

const getNodeIcon = (node: NodeType) => {
  return nodeTypeIconMap[node] ?? DEFAULT_NODE_ICON
}

export { getNodeIcon }
