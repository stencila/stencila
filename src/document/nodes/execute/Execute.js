import BlockNode from 'substance/model/BlockNode'

class Execute extends BlockNode {

  refresh () {
    if (this.source) {
      try {
        this.document.execute(this.source)
        this.error = false
      } catch (error) {
        this.error = true
        throw error
      }
      this.emit('content:changed')
    }
  }

}

Execute.define({
  type: 'execute',
  language: { type: 'string', default: '' },
  show: { type: 'boolean', default: false },
  error: { type: 'string', optional: true },
  extra: { type: 'string', optional: true },
  source: { type: 'string', default: '' }
})

export default Execute
