import test from 'tape'

import merge from '../../../../src/js-context/functions/types/merge'

test('merge', t => { 
  t.deepEqual(merge({}, { a: 1, b: 2 }), { a: 1, b: 2 })
  t.deepEqual(merge({ a: 0 }, { a: 1, b: 2}), { a: 1, b: 2 })
  t.deepEqual(merge({ a: { b: 1, c: 2 } }, { a: { b: 'b' }, d: 'd' }), { a: { b: 'b', c: 2 }, d: 'd' })

  t.throws(() => merge(), /Unable to dispatch function call "merge\(\)"/)
  t.throws(() => merge(1, null), /Unable to dispatch function call "merge\(integer, null\)"/)

  t.end()
})
