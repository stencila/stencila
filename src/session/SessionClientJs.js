import stencilaJs from 'stencila-js'

import ComponentClient from '../component/ComponentClient'

/**
 * A session client for a local Javascript session
 *
 * This provides inteface compatability with other, remote, session types that
 * are accessed via `SessionClient` (in particular, returning a `Promise` from
 * a call to `execute`)
 */
class SessionClientJs extends ComponentClient {

  constructor () {
    super()
    this.impl = new stencilaJs.JsSession()
  }

  execute (code, inputs) {
    return Promise.resolve(this.impl.execute(
      code, inputs
    ))
  }

}

export default SessionClientJs
