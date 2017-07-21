import test from 'tape'

import Storer from '../../src/host/Storer'

test('Storer:readFile', t => {
  let s = new Storer()
  t.throws(s.readFile, 'Storer.readFile() must be implemented in derived class')
  t.end()
})

test('Storer:writeFile', t => {
  let s = new Storer()
  t.throws(s.writeFile, 'Storer.writeFile() must be implemented in derived class')
  t.end()
})

test('Storer:readDir', t => {
  let s = new Storer()
  t.throws(s.readDir, 'Storer.readDir() must be implemented in derived class')
  t.end()
})
