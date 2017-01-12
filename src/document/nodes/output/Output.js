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
    this.document.documentSession.transaction(tx => {
      tx.set([this.id, 'content'], content)
      this.emit('content:changed') // CHECK: I thought that tx.set would emit this event; seems not.
    })
  }

}

Output.define({
  type: 'output',

  value: {type: 'string', optional: true},
  format: {type: 'string', optional: true},

  content: {type: 'string', default: ''}
})

export default Output

