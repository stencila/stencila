import test from 'tape'
import { Configurator } from 'substance'

import { FunctionPackage, FunctionSchema } from '../../src/function'
import JsContext from '../../src/js-context/JsContext'

import testVFS from '../../tmp/test-vfs.js'

function loadFunction (path) {
  let xml = testVFS[path]
  let configurator = new Configurator()
  configurator.import(FunctionPackage)
  const importer = configurator.createImporter(FunctionSchema.getName())
  const func = importer.importDocument(xml)
  return func
}

function testFunction (path) {
  const func = loadFunction(path)
  let context = new JsContext()
  return func.testImplem('js', context)
}

testFunction('tests/function/fixtures/cos.fun.xml').then(results => {
  test('function', t => {
    for (let result of results) {
      t.notOk(result.errors)
      t.deepEqual(result.output, result.expected)
      t.pass(result)
    }
    t.end()
  })
})
