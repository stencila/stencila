import {BlockNode} from 'substance'

class Default extends BlockNode {}

Default.define({
  type: 'default',
  html: {type: 'string', default: ''}
})

export default Default
