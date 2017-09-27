import test from 'tape'

import { importFunctionDocument } from '../../src/function'
import JsContext from '../../src/contexts/JsContext'

import testVFS from '../../tmp/test-vfs.js'

function loadFunction (path) {
  // Main XML function definition
  let main = testVFS[path]
  // Source code for function implementations that are "included"
  // must be in the same directory, so find siblings
  let basename = path.substr(0, Math.max(path.lastIndexOf('/'), 0))
  let siblings = {}
  for (let path of Object.keys(testVFS)) {
    let match = path.match('^' + basename + '/([\\w\\.]+)')
    if (match) siblings[match[1]] = testVFS[path]
  }

  return importFunctionDocument(main, siblings)
}

function testFunction (path) {
  const func = loadFunction(path)
  func.initialize()
  let context = new JsContext()
  return func.test('js', context)
}

for (let file of Object.keys(testVFS)) {
  if (file.match(/tests\/function\/fixtures\/(\w)+\.fun\.xml/)) {
    testFunction(file).then(results => {
      test(file, t => {
        for (let result of results) {
          t.notOk(result.errors)
          t.deepEqual(result.output, result.expected)
        }
        t.end()
      })
    })
  }
}
