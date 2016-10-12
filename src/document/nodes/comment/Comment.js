import Container from 'substance/model/Container'

class Comment extends Container {}

Comment.define({
  type: 'comment',
  who: { type: 'string', default: '' },
  when: { type: 'string', default: '' },
  nodes: { type: ['id'], default: [] }
})

export default Comment
