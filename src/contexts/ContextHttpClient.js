import {PUT} from '../util/requests'
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

  constructor(url) {
    super()
    this.url = url
  }

  /**
   * Get the list of supported programming languages
   *
   * @override
   */
  supportedLanguages () {
    return PUT(this.url + '!supportedLanguages')
  }

  /**
   * Get a list of function libraries
   *
   * @override
   */
  getLibraries () {
    return PUT(this.url + '!getLibraries')
  }

  /**
   * Analyse code
   *
   * @override
   */
  _analyseCode (code, exprOnly = false) {
    return PUT(this.url + '!analyseCode', {code: code, exprOnly: exprOnly})
  }

  /**
   * Execute code
   *
   * @override
   */
  _executeCode (code, inputs, exprOnly = false) {
    return PUT(this.url + '!executeCode', {code: code, inputs: inputs, exprOnly: exprOnly})
  }


  /**
   * Does the context provide a function?
   *
   * @override
   */
  hasFunction (name) {
    return PUT(this.url + '!hasFunction', {name: name})
  }

  /**
   * Call a function
   *
   * @override
   */
  callFunction (library, name, args, namedArgs) {
    return PUT(this.url + '!callFunction', {library: library, name: name, args: args, namedArgs: namedArgs})
  }
}
