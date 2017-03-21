import test from 'tape'

import _multimethod from '../../../../src/js-context/functions/types/_multimethod'

test('_multimethod', t => {
  let m1 = _multimethod('m1', {
    null: () => 'dispatched to null',
    integer: () => 'dispatched to integer',
    foo: () => 'dispatched to foo'
  }, () => 'dispatched to default')

  t.equal(m1(null), 'dispatched to null')
  t.equal(m1({type: 'foo'}), 'dispatched to foo')
  t.equal(m1(3.14), 'dispatched to default')

  let m2 = _multimethod('m2', {
    'null, null': () => 'dispatched to null, null',
    'integer, float': () => 'dispatched to integer, float',
    'foo, integer': () => 'dispatched to foo, integer'
  }, 2)

  t.equal(m2(null, null), 'dispatched to null, null')
  t.equal(m2(1, 3.14), 'dispatched to integer, float')
  t.equal(m2({type: 'foo'}, 42), 'dispatched to foo, integer')  
  t.throws(() => m2(), /Unable to dispatch call to "m2" with type\(s\) ""/)
  t.throws(() => m2(3.14), /Unable to dispatch call to "m2" with type\(s\) "float"/)

  t.end()
})
