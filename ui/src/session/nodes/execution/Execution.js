import {DocumentNode} from 'substance'

class Execution extends DocumentNode {}

Execution.define({
  type: 'execution',
  code: { type: 'string', default: '' },
  result: { type: 'object', default: {
    errors: null,
    output: null
  }}
})

export default Execution
