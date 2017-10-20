import test from 'tape'
import MiniContext from '../../src/contexts/MiniContext'
import FunctionManager from '../../src/function/FunctionManager'
import JsContext from '../../src/contexts/JsContext'
import { libtestXML, libtest } from './libtest'

function setupContextWithFunctions() {
  let functionManager = new FunctionManager()
  functionManager.importLibrary('test', libtestXML)
  let jsContext = new JsContext()
  jsContext._libs['test'] = libtest
  return new MiniContext(functionManager, { js: jsContext })
}

test('MiniContext: analyseCode(x=5)', t => {
  let c = new MiniContext()
  t.plan(1)
  c.analyseCode('x=5').then((res) => {
    t.equal(res.output, 'x', 'there should be output variable x')
  })
})

test('MiniContext: analyseCode(foo(x,y,z))', t => {
  let c = new MiniContext()
  t.plan(1)
  c.analyseCode('foo(x,y,z)').then((res) => {
    t.deepEqual(res.inputs, ['x','y','z'], 'there should be input variables x,y,z')
  })
})

test('MiniContext: x=5', t => {
  let c = new MiniContext()
  t.plan(1)
  c.executeCode('x=5').then((res) => {
    t.equal(res.data, 5, 'value should be correct')
  })
})

test('MiniContext: 1+2+3', t => {
  let c = new MiniContext()
  t.plan(1)
  c.executeCode('1+2+3').then((res) => {
    t.equal(res.data, 6, 'value should be correct')
  })
})

test('MiniContext: no_params()', t => {
  let c = setupContextWithFunctions()
  c.executeCode('no_params()').then((res) => {
    t.equal(res.type, 'integer', 'type should be correct')
    t.equal(res.data, 5, 'result should be correct')
    t.end()
  })
})

test('MiniContext: no_params() + 1', t => {
  let c = setupContextWithFunctions()
  c.executeCode('no_params() + 1').then((res) => {
    t.equal(res.type, 'integer', 'type should be correct')
    t.equal(res.data, 6, 'result should be correct')
    t.end()
  })
})

test('MiniContext: one_param(2)', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param(2)').then((res) => {
    t.equal(res.type, 'number', 'type should be correct')
    t.equal(res.data, 2.2, 'result should be correct')
    t.end()
  })
})

test('MiniContext: one_param()', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param()').then((res) => {
    t.ok(res instanceof Error, 'should error')
    t.equal(res.message, 'Required parameter "param1" was not supplied', 'error message should be correct')
    t.end()
  })
})

test('MiniContext: one_param(1, 2, 3)', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param(1, 2, 3)').then((res) => {
    t.ok(res instanceof Error, 'should error')
    t.equal(res.message, 'Too many parameters supplied (3), expected 1 at most', 'error message should be correct')
    t.end()
  })
})

test('MiniContext: one_param_with_default("Howdy!")', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param_with_default("Howdy!")').then((res) => {
    t.equal(res.data, "Howdy!")
    t.end()
  })
})

test('MiniContext: one_param_with_default()', t => {
  let c = setupContextWithFunctions()
  c.executeCode('one_param_with_default()').then((res) => {
    t.equal(res.data, "Hello!")
    t.end()
  })
})
