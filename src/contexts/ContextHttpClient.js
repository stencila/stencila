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
    this.host = host
    this.peer = url
    this.name = name
  }

  /**
   * Get the list of supported programming languages
   *
   * @override
   */
  supportedLanguages () {
    return this.host._put(this.peer, '/' + this.name + '!supportedLanguages')
  }

  /**
   * Get a list of function libraries
   *
   * @override
   */
  getLibraries () {
    return this.host._put(this.peer, '/' + this.name + '!getLibraries')
  }

  /**
   * Analyse code
   *
   * @override
   */
  _analyseCode (code, exprOnly = false) {
    return this.host._put(this.peer, '/' + this.name + '!analyseCode', {code: code, exprOnly: exprOnly})
  }

  /**
   * Execute code
   *
   * @override
   */
  _executeCode (code, inputs, exprOnly = false) {
    return this.host._put(this.peer, '/' + this.name + '!executeCode', {code: code, inputs: inputs, exprOnly: exprOnly})
  }


  /**
   * Does the context provide a function?
   *
   * @override
   */
  hasFunction (name) {
    return this.host._put(this.peer, '/' + this.name + '!hasFunction', {name: name})
  }

  /**
   * Call a function
   *
   * @override
   */
  callFunction (library, name, args, namedArgs) {
    return this.host._put(this.peer, '/' + this.name + '!callFunction', {library: library, name: name, args: args, namedArgs: namedArgs})
  }
}
