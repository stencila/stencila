// import test from 'tape'

// import Engine from '../../src/engine/Engine'
// import FunctionManager from '../../src/function/FunctionManager'
// import JsContext from '../../src/contexts/JsContext'
// import MiniContext from '../../src/contexts/MiniContext'
// import { libtestXML, libtest } from '../contexts/libtest'

// test('Engine: foo bar', t => {
//   let { engine } = _setup()
//   t.end()
// })

// function _setup() {
//   let functionManager = new FunctionManager()
//   functionManager.importLibrary('test', libtestXML)
//   let contexts = {}
//   let host = {
//     createContext: function(language) {
//       switch(language) {

//       }
//       if (language !== 'js') throw new Error('This stub only creates JsContexts')
//       return Promise.resolve(jsContext)
//     },
//     functionManager
//   }
//   let jsContext = new JsContext()
//   jsContext.importLibrary('test', libtest)
//   let miniContext = new MiniContext(host)
//   contexts['js'] = jsContext
//   contexts['mini'] = miniContext
//   let engine = new Engine(host)
//   // HACK: don't let the engine run automatically by overriding the trigger
//   engine._triggerScheduler = function() {}

//   return { engine, host, contexts }
// }
