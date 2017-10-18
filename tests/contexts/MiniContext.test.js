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
    let val = res
    t.equal(val, 5, 'value should be correct')
  })
})

test('MiniContext: 1+2+3', t => {
  let c = new MiniContext()
  t.plan(1)
  c.executeCode('1+2+3').then((res) => {
    let val = res
    t.equal(val, 6, 'value should be correct')
  })
})

test('MiniContext: foo()', t => {
  let c = setupContextWithFunctions()
  t.plan(1)
  c.executeCode('foo()').then((res) => {
    let val = res
    t.equal(val, 'foo', 'result should be correct')
  })
})
