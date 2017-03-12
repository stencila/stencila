import { InlineNode } from 'substance'

class Math extends InlineNode {}

Math.define({
  type: 'math',
  source: { type: 'string', default: '' },
  language: { type: 'string', default: 'asciimath' }
})

export default Math
