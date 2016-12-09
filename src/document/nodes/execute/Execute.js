import BlockNode from 'substance/model/BlockNode'

class Execute extends BlockNode {

  refresh () {
    if (this.source) {
      this.document.session(this.language).then(session => {
        try {
          let timer = window.performance
          let t0 = timer.now()
          let args = this.document.variables
          session.execute(this.source, args).then(result => {
            this.duration = (timer.now() - t0) / 1000

            if (this.name) {
              this.document.setVariable(this.name, result.output)
            } else {
              this.result = result
              this.emit('changed')
            }
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
  name: { type: 'string', default: null },
  language: { type: 'string', default: '' },
  depends: { type: 'string', default: '' }, // Comma separated list, can it be an array?
  show: { type: 'boolean', default: false },
  extra: { type: 'string', optional: true },
  source: { type: 'string', default: '' },
  result: { type: 'object', default: {} },
  duration: { type: 'number', default: 0 }
})

export default Execute
