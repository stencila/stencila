import BlockNode from 'substance/model/BlockNode'

class Execute extends BlockNode {

  refresh () {
    if (this.code) {
      // Get the session
      this.document.session(this.session).then(session => {
        try {
          let timer = window.performance
          let t0 = timer.now()
          let inputs = this.document.variables
          // Call `session.execute()` with code and inputs
          session.execute(this.code, inputs).then(result => {
            this.duration = (timer.now() - t0) / 1000

            if (this.output) {
              this.document.setVariable(this.output, result.output)
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

  session: { type: 'string', default: '' },

  inputs: { type: 'object', default: {} },
  output: { type: 'string', default: '' },

  extra: { type: 'string', optional: true },

  code: { type: 'string', default: '' },

  result: { type: 'object', default: {} },
  duration: { type: 'number', default: 0 }
})

export default Execute
