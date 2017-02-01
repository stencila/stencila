import BlockNode from 'substance/model/BlockNode'

class Execute extends BlockNode {

  getCall () {
    let call = `${this.session}(${this.input})`
    if (this.output) call = `${this.output} = ${call}`
    return call
  }

  setCall (call) {
    let match = call.match(/(([\w_]+) *= *)?(\w+)\(([^(]*)\)/)
    if (match) {
      this.output = match[2]
      this.session = match[3]
      this.input = match[4]
    } else {
      throw new Error('Invalid format for call')
    }
  }

  /**
   * Refresh this directive by executing code
   *
   * The code is executed in an appropriate session
   */
  refresh () {
    if (this.code) {
      // Get the session
      this.document.getSession(this.session).then(session => {
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
            this.result = {}
          } else {
            this.result = result
          }
          this.emit('changed')
        })
      }).catch(error => {
        this.result = {
          errors: {
            '0': error.toString()
          }
        }
        this.emit('changed')
        throw error
      })
    }
  }

}

Execute.define({
  type: 'execute',

  session: { type: 'string', default: '' },
  input: { type: 'string', default: '' },
  output: { type: 'string', default: '' },
  show: { type: 'bool', default: false },
  extra: { type: 'string', optional: true },
  code: { type: 'string', default: '' },

  result: { type: 'object', default: {} },
  duration: { type: 'number', default: 0 }
})

export default Execute
