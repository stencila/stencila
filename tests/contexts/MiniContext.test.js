import test from 'tape'
import MiniContext from '../../src/contexts/MiniContext'
import JsContext from '../../src/contexts/JsContext'
import FunctionManager from '../../src/function/FunctionManager'
import { libtest } from './libtest'

function setupContextWithFunctions() {
  // A JsContext with the test function library
  let jsContext = new JsContext()
  jsContext.importLibrary(libtest)
  // Function manager for getting function specs
  let functionManager = new FunctionManager()
  functionManager.importLibrary(jsContext, libtest)
  // A mock Host that provides the JsContext when requested
  let host = {
    createContext: function(language) {
      if (language !== 'js') throw new Error('This stub only creates JsContexts')
      return Promise.resolve(jsContext)
    },
    functionManager
  }
  return new MiniContext(host)
}

test('MiniContext: analyseCode(x=5)', t => {
  let c = setupContextWithFunctions()
  t.plan(1)
  c.analyseCode('x=5').then((res) => {
    t.equal(res.output, 'x', 'there should be output variable x')
  })
})

test('MiniContext: analyseCode(foo(x,y,z))', t => {
  let c = setupContextWithFunctions()
  t.plan(1)
  c.analyseCode('foo(x,y,z)').then((res) => {
    t.deepEqual(res.inputs, ['x','y','z'], 'there should be input variables x,y,z')
  })
})

test('MiniContext: x=5', t => {
  let c = setupContextWithFunctions()
  t.plan(1)
  c.executeCode('x=5').then((res) => {
    t.equal(res.value.data, 5, 'value should be correct')
  })
})

test('MiniContext: 1+2+3', t => {
  let c = setupContextWithFunctions()
  t.plan(1)
  c.executeCode('1+2+3').then((res) => {
    let val = res.value
    t.equal(val.data, 6, 'value should be correct')
  })
})

test('MiniContext: no_params()', t => {
  let c = setupContextWithFunctions()
  c.executeCode('no_params()').then((res) => {
    let val = res.value
    t.equal(val.type, 'integer', 'type should be correct')
    t.equal(val.data, 5, 'result should be correct')
    t.end()
  })
})

test('MiniContext: no_params() + 1', t => {
  let c = setupContextWithFunctions()
  c.executeCode('no_params() + 1').then((res) => {
    let val = res.value
    t.equal(val.type, 'integer', 'type should be correct')
    t.equal(val.data, 6, 'result should be correct')
    t.end()
  })
})

test('MiniContext: one_param(2)', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param(2)').then((res) => {
    let val = res.value
    t.equal(val.type, 'number', 'type should be correct')
    t.equal(val.data, 2.2, 'result should be correct')
    t.end()
  })
})


test('MiniContext: one_param_with_default("Howdy!")', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param_with_default("Howdy!")').then((res) => {
    t.equal(res.value.data, "Howdy!")
    t.end()
  })
})

test('MiniContext: one_param_with_default()', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param_with_default()').then((res) => {
    t.equal(res.value.data, "Hello!")
    t.end()
  })
})
