import {BlockNode} from 'substance'

class Session extends BlockNode {

}

Session.define({
  type: 'session',
  url: { type: 'string', default: '' },
  language: { type: 'string', default: '' }
})

export default Session
