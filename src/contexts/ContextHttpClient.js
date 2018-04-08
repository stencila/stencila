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
   * Analyse code
   *
   * @override
   */
  _analyseCode (code, exprOnly = false) {
    let before = {
      type: exprOnly ? 'expr' : 'block',
      source: {
        type: 'text',
        data: code
      },
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
    let before = {
      type: exprOnly ? 'expr' : 'block',
      source: {
        type: 'text',
        data: code
      },
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
}
