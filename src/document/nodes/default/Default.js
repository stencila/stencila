import BlockNode from 'substance/model/BlockNode'

class Default extends BlockNode {}

Default.define({
  type: 'default',
  html: {type: 'string', default: ''}
})

export default Default
