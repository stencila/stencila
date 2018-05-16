import { JavascriptContext } from 'stencila-js'
import ContextClient from './ContextClient'

export default class JavascriptContextClient extends ContextClient {

  constructor(host) {
    super(host)

    this._jsContext = new JavascriptContext(host, 'js')
  }

  // TODO: not sure if this should be API
  importLibrary(lib) {
    // TODO: take a look how libcore looks like
    this._jsContext._libraries[lib.name] = lib
    return this
  }

  _libraries() {
    return Promise.resolve(this._jsContext.libraries())
  }

  _compile(cell) {
    return this._jsContext.compile(cell)
  }

  _execute(cell) {
    return this._jsContext.execute(cell)
  }

  _evaluate(call) {
    return this._jsContext.evaluate(call)
  }

}