import BlockNode from 'substance/model/BlockNode'

class Execute extends BlockNode {

  refresh () {
    if (this.source) {
      this.document.session(this.language).then(session => {
        try {
          let timer = window.performance
          let t0 = timer.now()
          session.execute(this.source, this.document.pipeline).then(result => {
            this.duration = (timer.now() - t0) / 1000
            this.result = result

            // Update the pipline with the pipes from the session
            this.document.pipeline = {}
            result.pipes.forEach(name => {
              this.document.pipeline[name] = session._url
            })

            this.emit('changed')
          })
        } catch (error) {
          this.result = {
            errors: {
              '0': error.toString()
            }
          }
          this.emit('changed')
          throw error
        }
      })
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
