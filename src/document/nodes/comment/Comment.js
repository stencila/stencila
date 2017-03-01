import {Container} from 'substance'

class Comment extends Container {}

Comment.define({
  type: 'comment',
  who: { type: 'string', default: '' },
  when: { type: 'string', default: '' },
  nodes: { type: ['id'], default: [] }
})

export default Comment
