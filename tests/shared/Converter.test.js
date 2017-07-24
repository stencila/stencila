import test from 'tape'

import Converter from '../../src/shared/Converter'

test('Converter:match', t => {
  let c = new Converter()
  t.throws(c.match, 'Converter.match() must be implemented in derived class')
  t.end()
})

test('Converter:import', t => {
  let c = new Converter()
  t.throws(c.import, 'Converter.import() must be implemented in derived class')
  t.end()
})

test('Converter:export', t => {
  let c = new Converter()
  t.throws(c.export, 'Converter.export() must be implemented in derived class')
  t.end()
})

test('Converter:parsePath', t => {
  let c = new Converter()
  t.deepEqual(c._parsePath('file.ext'), {dir: '', file: 'file.ext', ext: 'ext'})
  t.deepEqual(c._parsePath('./file.ext'), {dir: '.', file: 'file.ext', ext: 'ext'})
  t.deepEqual(c._parsePath('a/file.ext'), {dir: 'a', file: 'file.ext', ext: 'ext'})
  t.deepEqual(c._parsePath('/a/file.ext'), {dir: '/a', file: 'file.ext', ext: 'ext'})
  t.deepEqual(c._parsePath('a/b/c/file.ext'), {dir: 'a/b/c', file: 'file.ext', ext: 'ext'})
  t.deepEqual(c._parsePath('a/b/c'), {dir: 'a/b/c', file: null, ext: null})
  t.end()
})
