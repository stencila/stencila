import BlockNode from 'substance/model/BlockNode'

class Session extends BlockNode {

}

Session.define({
  type: 'session',
  url: { type: 'string', default: '' }
})

export default Session
