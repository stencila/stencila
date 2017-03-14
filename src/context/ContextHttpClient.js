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
   * Run code
   * 
   * @override
   */
  runCode (code) {
    return PUT(this.url + '!runCode', {code: code})
  }

  /**
   * Call code
   * 
   * @override
   */
  callCode (code, args) {
    return PUT(this.url + '!callCode', {code: code, args: args})
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
  callFunction (name, args, options) {
    return PUT(this.url + '!callFunction', {name: name, args: args, options: options})
  }

  /**
   * Get the dependencies for a piece of code
   *
   * @override
   */
  codeDependencies (code) {
    return PUT(this.url + '!codeDependencies', {code: code})
  }

  /**
   * Complete a piece of code
   *
   * @override
   */
  codeComplete (code) {
    return PUT(this.url + '!codeComplete', {code: code})
  }
}
