/**
 * A HTTP client for a remote `Context`
 *
 * Implements the `Context` API by remote procedure calls (RPC) to a remote
 * context (e.g. a `RContext` running in a different process)
 *
 * @extends Context
 */
export default class ContextClient {

  constructor(host) {
    this._host = host
  }

  libraries() {
    return this._libraries()
  }

  analyseCode(code, exprOnly = false) { // eslint-disable-line no-unused-vars
    let cell = {
      type: 'cell',
      source: {
        type: 'text',
        data: code
      },
      expr: exprOnly
    }
    return this._compile(cell)
    .then(res => {
      let inputs = res.inputs && res.inputs.map(input => input.name)
      let output = res.outputs && res.outputs[0] && res.outputs[0].name
      let messages = res.messages
      return { inputs, output, messages }
    })
  }

  executeCode(code, inputs, exprOnly = false) {
    let cell = {
      type: 'cell',
      source: {
        type: 'text',
        data: code
      },
      expr: exprOnly,
      inputs: Object.entries(inputs).map(([name, value]) => {
        return {name, value}
      })
    }
    return this._execute(cell)
    .then(res => {
      let inputs = res.inputs && res.inputs.map(input => input.name)
      let output = res.outputs && res.outputs[0] && res.outputs[0].name
      let value = res.outputs && res.outputs[0] && res.outputs[0].value
      let messages = res.messages
      if (value) {
        if (value.type === 'library') {
          this._host._functionManager.importLibrary(this, value.data)
        } else if (value.type === 'function') {
          this._host._functionManager.importFunction(this, value.data)
        }
      }
      return { inputs, output, value, messages }
    })
  }

  callFunction(library, name, args, namedArgs) {
    let call = {
      type: 'call',
      func: {type: 'get', name: name},
      args, namedArgs
    }
    return this._evaluate(call)
  }

  _libraries() {
    throw new Error('This method is abstract.')
  }

  _compile(cell) { // eslint-disable-line no-unused-vars
    throw new Error('This method is abstract.')
  }

  _execute(cell) { // eslint-disable-line no-unused-vars
    throw new Error('This method is abstract.')
  }

  _evaluate(cell) { // eslint-disable-line no-unused-vars
    throw new Error('This method is abstract.')
  }

}
