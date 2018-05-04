import Engine from '../../src/engine/Engine'
import JsContext from '../../src/contexts/JsContext'
import MiniContext from '../../src/contexts/MiniContext'
import FunctionManager from '../../src/function/FunctionManager'
import { libtest } from '../contexts/libtest'

export default function setupEngine() {
  // A JsContext with the test function library
  let jsContext = new JsContext()
  let miniContext
  jsContext.importLibrary(libtest)
  // Function manager for getting function specs
  let functionManager = new FunctionManager()
  functionManager.importLibrary(jsContext, libtest)
  // A mock Host that provides the JsContext when requested
  let host = {
    _disable(val) {
      this._disabled = val
    },
    createContext: function(lang) {
      if (this._disabled) {
        return Promise.resolve(new Error('No context for language '+lang))
      }
      switch (lang) {
        case 'js':
          return Promise.resolve(jsContext)
        case 'mini':
          return Promise.resolve(miniContext)
        default:
          return Promise.resolve(new Error('No context for language '+lang))
      }
    },
    functionManager
  }
  miniContext = new MiniContext(host)
  host.engine = new Engine({ host })
  let engine = host.engine
  let graph = engine._graph
  // don't let the engine be run forever in tests
  engine.run = () => {}
  return { host, engine, graph }
}