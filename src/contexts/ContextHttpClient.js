import ContextClient from './ContextClient'

/**
 * A HTTP client for a remote `Context`
 *
 * Implements the `Context` API by remote procedure calls (RPC) to a remote
 * context (e.g. a `RContext` running in a different process)
 *
 * @extends Context
 */
export default class ContextHttpClient extends ContextClient {

  constructor (host, url, name) {
    super(host)

    this._peer = url
    this._name = name
  }

  /**
   * Get a list of libraries
   */
  _libraries () {
    return this._host._put(this._peer, '/' + this._name + '!libraries')
  }

  _compile (pre) {
    return this._host._put(this._peer, '/' + this._name + '!compile', pre)
  }

  _execute (pre) {
    return this._host._put(this._peer, '/' + this._name + '!execute', pre)
  }

  _evaluate (call) {
    return this._host._put(this._peer, '/' + this._name + '!evaluate', call)
  }

}
