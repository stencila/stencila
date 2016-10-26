import BlockNode from 'substance/model/BlockNode'

class Execute extends BlockNode {

  refresh () {
    if (this.source) {
      // Check list of sessions and open a new session if necessary
      if (!this.document.session) {
        this.document.host.new('session-' + this.language).then(session => {
          this.document.session = session
          this.refresh()
        })
      } else {
        try {
          let timer = window.performance
          let t0 = timer.now()
          this.document.session.execute(this.source).then(result => {
            this.duration = (timer.now() - t0) / 1000
            this.result = result
          })
        } catch (error) {
          this.result = {
            errors: {
              '0': error.toString()
            }
          }
          throw error
        }
        this.emit('changed')
      }
    }
  }

}

Execute.define({
  type: 'execute',
  language: { type: 'string', default: '' },
  show: { type: 'boolean', default: false },
  extra: { type: 'string', optional: true },
  source: { type: 'string', default: '' },
  result: { type: 'object', default: {} },
  duration: { type: 'number', default: 0 }
})

export default Execute
