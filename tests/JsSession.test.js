const {pack} = require('../src/packing')
const JsSession = require('../src/JsSession')

const test = require('tape')

test('JsSession.execute with no inputs, no errors and no output', function (t) {
  let s = new JsSession()

  t.deepEqual(s.execute('let x = 3\n'), {errors: {}, output: null}, 'assign')
  t.end()
})

test('JsSession.execute with no inputs, no errors', function (t) {
  let s = new JsSession()

  t.deepEqual(s.execute('42'), {errors: {}, output: pack(42)}, 'just an evaluation')
  t.deepEqual(s.execute('let x = 3\nx*3'), {errors: {}, output: pack(9)}, 'assign and return')
  t.end()
})

test('JsSession.execute with inputs and outputs but no errors', function (t) {
  let s = new JsSession()

  t.deepEqual(s.execute('a*6', {a: pack(7)}), {errors: {}, output: pack(42)})
  t.deepEqual(s.execute('a*b[1]', {a: pack(17), b: pack([1, 2, 3])}), {errors: {}, output: pack(34)})
  t.end()
})

test('JsSession.execute with errors', function (t) {
  let s = new JsSession()

  t.deepEqual(s.execute('foo'), {errors: { 1: 'ReferenceError: foo is not defined' }, output: null})
  t.deepEqual(s.execute('1\n2\nfoo\n4'), {errors: { 3: 'ReferenceError: foo is not defined' }, output: null})
  t.deepEqual(s.execute('for'), {errors: { 0: 'SyntaxError: Unexpected token for' }, output: null})
  t.end()
})
