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
      type: this._parseType(code),
      source: {
        type: 'text',
        data: code
      },
      expr: exprOnly,
      inputs: [],
      output: {},
      messages: []
    }
    return this._host._put(this._peer, '/' + this._name + '!compile', pre).then(post => {
      return {
        inputs: post.inputs && post.inputs.map(input => input.name),
        output: (post.output && post.output.name) || null,
        value: post.output && post.output.value,
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
    const type = this._parseType(code)
    let pre = {
      type,
      source: {
        type: 'text',
        data: code
      }
    }
    if (type === 'cell') {
      pre.expr = exprOnly
      pre.inputs = Object.entries(inputs).map(([name, value]) => {
        return {name, value}
      })
    }
    return this._host._put(this._peer, '/' + this._name + '!execute', pre).then(post => {
      if (post.type === 'library') {
        if (!post.status) this._host._functionManager.importLibrary(this, post)
        return post
      } else if (post.type === 'func') {
        if (!post.status) this._host._functionManager.importFunction(this, 'local', post)
        return post
      } else {
        return {
          inputs: post.inputs && post.inputs.map(input => input.name),
          output: (post.output && post.output.name) || null,
          value: post.output && post.output.value,
          messages: post.messages
        }
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

  _parseType(code) {
    const match = code.match(/^(\/\/|#|--)\s*!\s*(\w+)\s*/)
    if (match) {
      return match[2]
    } else {
      return 'cell'
    }
  }
}
