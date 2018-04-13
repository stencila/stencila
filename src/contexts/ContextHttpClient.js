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
    let before = {
      type: 'cell',
      source: {
        type: 'text',
        data: code
      },
      expr: exprOnly,
      inputs: [],
      output: {},
      messages: []
    }
    return this._host._put(this._peer, '/' + this._name + '!compile', before).then(after => {
      return {
        inputs: after.inputs.map(input => input.name),
        output: after.output.name || null,
        value: after.output.value,
        messages: after.messages
      }      
    })
  }

  /**
   * Execute code
   *
   * @override
   */
  _executeCode (code, inputs, exprOnly = false) {
    const match = code.match(/^(\/\/|#|--)!\s*(\w+)(\s+(.*))?$/)
    if (match) {
      const command = match[2]
      const arg = match[4]
      if (command === 'library') {
        return this._host._put(this._peer, '/' + this._name + '!executeLibrary', arg).then(result => {
          if (!result.messages) this._host._functionManager.importLibrary(this, result)
          return {
            value: null,
            messages: result.messages
          }
        })
      }
    }

    let before = {
      type: 'cell',
      source: {
        type: 'text',
        data: code
      },
      expr: exprOnly,
      inputs: Object.entries(inputs).map(([name, value]) => {
        return {name, value}
      }),
      output: {},
      messages: []
    }
    return this._host._put(this._peer, '/' + this._name + '!execute', before).then(after => {
      return {
        inputs: after.inputs.map(input => input.name),
        output: after.output.name || null,
        value: after.output.value,
        messages: after.messages
      }
    })
  }

  callFunction (library, name, args, namedArgs) {
    let call = {
      type: 'call',
      func: {type: 'get', name: name, from: {type: 'get', name: library}},
      args, namedArgs
    }
    return this._host._put(this._peer, '/' + this._name + '!execute', call)
  }
}
