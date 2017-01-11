import BlockNode from 'substance/model/BlockNode'

class Execute extends BlockNode {

  /**
   * Refresh this directive by executing code
   *
   * The code is executed in an appropriate session
   */
  refresh () {
    if (this.code) {
      // Get the session
      this.document.getSession(this.session).then(session => {
        try {
          let timer = window.performance
          let t0 = timer.now()
          // Pack input for sending
          let inputs = {}
          for (let variable of this.input.split(',')) {
            let pack = this.document.variables[variable]
            if (pack) inputs[variable] = pack
          }
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
  input: { type: 'string', default: '' },
  output: { type: 'string', default: '' },
  extra: { type: 'string', optional: true },
  code: { type: 'string', default: '' },

  result: { type: 'object', default: {} },
  duration: { type: 'number', default: 0 }
})

export default Execute
