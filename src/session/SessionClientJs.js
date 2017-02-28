import { JsContext } from 'stencila-js'

import ComponentDelegate from '../component/ComponentDelegate'

/**
 * A session client for a local Javascript session
 *
 * This provides inteface compatability with other, remote, session types that
 * are accessed via `SessionClient` (in particular, returning a `Promise` from
 * a call to `execute`)
 */
class SessionClientJs extends ComponentDelegate {

  constructor () {
    super()
    this.impl = new stencilaJs.JsContext()
  }

  execute (code, inputs) {
    return Promise.resolve(this.impl.execute(
      code, inputs
    ))
  }

}

export default SessionClientJs
