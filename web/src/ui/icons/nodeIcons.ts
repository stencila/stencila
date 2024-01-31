import { NodeType } from '../../types'

const DEFAULT_NODE_ICON = 'text'

const nodeTypeIconMap: { [k: NodeType]: string } = {
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
