import InlineNode from 'substance/model/InlineNode'

class Math extends InlineNode {}

Math.define({
  type: 'math',
  source: { type: 'string', default: '' },
  language: { type: 'string', default: 'asciimath' },
  display: { type: 'string', default: 'inline' },
  error: { type: 'string', optional: true }
})

export default Math
