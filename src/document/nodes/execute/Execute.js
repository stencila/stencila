import { BlockNode } from 'substance'
import { pack, unpack } from 'stencila-js'

class Execute extends BlockNode {

  getCall () {
    let call = `${this.context}(${this.input})`
    if (this.output) call = `${this.output} = ${call}`
    return call
  }

  setCall (call) {
    let match = call.match(/(([\w_]+) *= *)?(\w+)\(([^(]*)\)/)
    if (match) {
      this.output = match[2]
      this.context = match[3]
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
      // Get the context
      this.document.getSession(this.context).then(context => {
        let timer = window.performance
        let t0 = timer.now()
        // Pack input for sending
        let inputs = {}
        for (let variable of this.input.split(',')) {
          let value = this.document.variables[variable.trim()]
          if (value) inputs[variable] = pack(value)
        }
        // Call `context.execute()` with code and inputs...
        context.execute(this.code, inputs).then(returned => {
          this.duration = (timer.now() - t0) / 1000
          this.errors = returned.errors
          if (returned.output) {
            this.results = [returned.output]
          } else {
            this.results = []
          }
          this.emit('changed')
          // If this execute has an output variable then set it
          if (this.output) {
            this.document.setVariable(this.output, unpack(returned.output))
          }
        })
      }).catch(error => {
        this.errors = error.toString()
        this.results = null
        this.emit('changed')
        throw error
      })
    }
  }

}

Execute.define({
  type: 'execute',

  context: { type: 'string', default: '' },
  input: { type: 'string', default: '' },
  output: { type: 'string', default: '' },
  show: { type: 'bool', default: false },
  extra: { type: 'string', optional: true },
  code: { type: 'string', default: '' },

  errors: { type: 'object', optional: true },
  results: { type: ['array', 'object'], optional: true },
  duration: { type: 'number', default: 0 }
})

export default Execute
