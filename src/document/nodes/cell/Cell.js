import { BlockNode } from 'substance'

class Cell extends BlockNode {
}

Cell.schema = {
  type: 'cell',
  expression: { type: 'string', default: '' },
  language: { type: 'string', optional: true },
  sourceCode: { type: 'string', optional: true },
  output: { type: 'string', optional: true }
}

export default Cell
