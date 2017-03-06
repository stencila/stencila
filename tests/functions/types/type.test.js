import test from 'tape'

import type from '../../../src/functions/types/type'

test('type()', t => {
  t.equal(type(null), 'null')

  t.equal(type(true), 'bool')
  t.equal(type(false), 'bool')

  t.equal(type(42), 'int')
  t.equal(type(1000000000), 'int')
  t.equal(type(1.1e20), 'int')

  t.equal(type(3.14), 'flt')
  t.equal(type(Math.PI), 'flt')
  t.equal(type(1.1e-20), 'flt')

  t.equal(type(''), 'str')
  t.equal(type('Yo!'), 'str')

  t.equal(type([]), 'arr')
  t.equal(type([1, 2, 3]), 'arr')

  t.equal(type({}), 'obj')
  t.equal(type({a: 1, b: 2}), 'obj')

  t.equal(type([{}]), 'tab')
  t.equal(type([{a: 1, b: 2}]), 'tab')
  t.equal(type([{a: 1, b: 2}, 'non-an-object']), 'arr')

  t.end()
})
