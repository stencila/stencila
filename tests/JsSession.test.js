const {pack} = require('../src/packing')
const JsSession = require('../src/JsSession')

const test = require('tape')

test('JsSession can be constructed with options', t => {
  let s1 = new JsSession()
  let s2 = new JsSession({
    transform: true
  })

  t.equal(s1.options.transform, typeof window !== 'undefined', 'transform defaults to true in browser, false otherwise')
  t.equal(s2.options.transform, true)

  t.end()
})

test('JsSession.execute with no inputs, no errors and no output', function (t) {
  let s = new JsSession()

  t.deepEqual(s.execute('let x = 3\n\n'), {errors: {}, output: null}, 'assign')

  t.deepEqual(s.execute('// Multiple lines and comments\nlet x = {\na:1\n\n}\n\n'), {errors: {}, output: null}, 'assign')

  t.end()
})

test('JsSession.execute with no inputs, no errors', function (t) {
  let s = new JsSession()

  t.deepEqual(s.execute('42'), {errors: {}, output: pack(42)}, 'just an evaluation')
  t.deepEqual(s.execute('let x = 3\nx*3'), {errors: {}, output: pack(9)}, 'assign and return')
  t.deepEqual(s.execute('let x = 3\nx\n\n'), {errors: {}, output: null}, 'empty last line so no output')
  t.deepEqual(s.execute('42\n'), {errors: {}, output: pack(42)}, 'trailing newline ignores, so output')
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

test('JsSession has globals', function (t) {
  let s = new JsSession()

  s.execute('globals.foo = 42')
  t.equal(s.globals.foo, 42, 'can assign from execute')

  t.deepEqual(s.execute('globals.foo'), {errors: {}, output: pack(42)}, 'can access from execute')

  t.deepEqual(s.execute('foo'), s.execute('globals.foo'), 'can acess from execute directly (scoped within globals)')

  t.deepEqual(s.execute('foo', {foo: pack('bar')}), {errors: {}, output: pack('bar')}, 'inputs (locals) mask globals')

  t.deepEqual(s.execute('foo'), s.execute('globals.foo'), 'inputs only mask globals per call')

  t.end()
})

test('JsSession will transform code to ES2015(ES6)', function (t) {
  let s = new JsSession({
    transform: true
  })

  t.deepEqual(s.execute('Math.max(...[1,3,2])'), {errors: {}, output: pack(3)})

  t.end()
})

if (typeof window !== 'undefined') {
  test('JsSession can dynamically require NPM modules', t => {
    let s = new JsSession()

    t.deepEqual(s.execute('let isNumber = require("is-number")\nisNumber(1)'), {errors: {}, output: pack(true)})

    t.end()
  })
}
