import BlockNode from 'substance/model/BlockNode'

class Codeblock extends BlockNode {}

Codeblock.define({
  type: 'codeblock',
  language: { type: 'string', default: '' },
  source: { type: 'string', default: '' }
})

export default Codeblock

