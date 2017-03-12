import {BlockNode} from 'substance'

class Codeblock extends BlockNode {}

Codeblock.define({
  type: 'codeblock',
  language: { type: 'string', default: 'text' },
  source: { type: 'string', default: '' }
})

export default Codeblock

