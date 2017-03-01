import {Container} from 'substance'

class Discussion extends Container {}

Discussion.define({
  type: 'discussion',
  status: { type: 'string', default: 'open' },
  nodes: { type: ['id'], default: [] }
})

export default Discussion
