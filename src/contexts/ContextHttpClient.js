import Context from './Context'

/**
 * A HTTP client for a remote `Context`
 *
 * Implements the `Context` API by remote procedure calls (RPC) to a remote
 * context (e.g. a `RContext` running in a different process)
 *
 * @extends Context
 */
export default class ContextHttpClient extends Context {

  constructor(host, url, name) {
    super()
    this._host = host
    this._peer = url
    this._name = name
  }

  /**
   * Get a list of libraries
   */
  libraries () {
    return this._host._put(this._peer, '/' + this._name + '!libraries')
  }

  /**
   * Analyse code
   *
   * @override
   */
  _analyseCode (code, exprOnly = false) {
    let pre = {
      type: 'cell',
      source: {
        type: 'text',
        data: code
      },
      expr: exprOnly
    }
    return this._host._put(this._peer, '/' + this._name + '!compile', pre).then(post => {
      return {
        inputs: post.inputs && post.inputs.map(input => input.name),
        output: post.outputs && post.outputs[0] && post.outputs[0].name,
        messages: post.messages
      }      
    })
  }

  /**
   * Execute code
   *
   * @override
   */
  _executeCode (code, inputs, exprOnly = false) {
    let pre = {
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
    return this._host._put(this._peer, '/' + this._name + '!execute', pre).then(post => {
      let output = post.outputs && post.outputs[0] && post.outputs[0].name
      let value = post.outputs && post.outputs[0] && post.outputs[0].value
      if (value) {
        if (value.type === 'library') {
          this._host._functionManager.importLibrary(this, value)
        } else if (value.type === 'function') {
          this._host._functionManager.importFunction(this, value)
          value = { type: 'object', data: value }
        }
      }
      return {
        inputs: post.inputs && post.inputs.map(input => input.name),
        output: output,
        value: value,
        messages: post.messages
      }
    })
  }

  callFunction (library, name, args, namedArgs) {
    let call = {
      type: 'call',
      func: {type: 'get', name: name},
      args, namedArgs
    }
    return this._host._put(this._peer, '/' + this._name + '!evaluate', call)
  }
}
