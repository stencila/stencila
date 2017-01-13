import {sprintf} from 'sprintf-js'
import {unpack} from 'stencila-js'

import InlineNode from 'substance/model/InlineNode'

class Output extends InlineNode {

  refresh () {
    let content = ''
    let pack = this.document.getVariable(this.value)
    if (pack) {
      let value = unpack(pack)
      if (this.format) {
        content = sprintf(this.format, value)
      } else {
        content = value.toString()
      }
    }
    // CHECK
    // This is how I thought this should be done....
    // this.document.documentSession.transaction(tx => {
    //   tx.set([this.id, 'content'], content)
    // })
    // This is what works...
    this.content = content
    this.emit('content:changed')
  }

}

Output.define({
  type: 'output',

  value: {type: 'string', optional: true},
  format: {type: 'string', optional: true},

  content: {type: 'string', default: ''}
})

export default Output

