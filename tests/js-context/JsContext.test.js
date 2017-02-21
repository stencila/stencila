const {pack} = require('../../src/packing')
const JsContext = require('../../src/js-context/JsContext')

const test = require('tape')

test('JsContext can be constructed with options', t => {
  let c1 = new JsContext()
  let c2 = new JsContext({
    transform: true
  })

  t.equal(c1.options.transform, typeof window !== 'undefined', 'transform defaults to true in browser, false otherwise')
  t.equal(c2.options.transform, true)

  t.end()
})

test('JsContext.execute with no inputs, no errors and no output', function (t) {
  let c = new JsContext()

  t.deepEqual(c.execute('let x = 3\n\n'), {errors: null, output: null}, 'assign')

  t.deepEqual(c.execute('// Multiple lines and comments\nlet x = {\na:1\n\n}\n\n'), {errors: null, output: null}, 'assign')

  t.end()
})

test('JsContext.execute with no inputs, no errors', function (t) {
  let c = new JsContext()

  t.deepEqual(c.execute('return 42'), {errors: null, output: pack(42)}, 'just an evaluation')
  t.deepEqual(c.execute('let x = 3\nreturn x*3'), {errors: null, output: pack(9)}, 'assign and return')
  t.deepEqual(c.execute('let x = 3\nx*3\n'), {errors: null, output: null}, 'no return so no output')
  t.end()
})

test('JsContext.execute with inputs and outputs but no errors', function (t) {
  let c = new JsContext()

  t.deepEqual(c.execute('return a*6', {a: pack(7)}), {errors: null, output: pack(42)})
  t.deepEqual(c.execute('return a*b[1]', {a: pack(17), b: pack([1, 2, 3])}), {errors: null, output: pack(34)})
  t.end()
})

test('JsContext.execute output multiline', function (t) {
  let c = new JsContext()

  t.deepEqual(c.execute(`return {
    jermaine: 'Hiphopopotamus',
    brett: 'Rhymnoceros'
  }`, null, {pack: false}), {errors: null, output: { brett: 'Rhymnoceros', jermaine: 'Hiphopopotamus' }})
  t.end()
})

test('JsContext.execute with errors', function (t) {
  let c = new JsContext()

  t.deepEqual(c.execute('foo'), {errors: { 1: 'ReferenceError: foo is not defined' }, output: null})
  t.deepEqual(c.execute('1\n2\nfoo\n4'), {errors: { 3: 'ReferenceError: foo is not defined' }, output: null})
  t.deepEqual(c.execute('<>'), {errors: { 0: 'SyntaxError: Unexpected token <' }, output: null})
  t.end()
})

test('JsContext has globals', function (t) {
  let c = new JsContext()

  c.execute('globals.foo = 42')
  t.equal(c.globals.foo, 42, 'can assign from execute')

  t.deepEqual(c.execute('return globals.foo'), {errors: null, output: pack(42)}, 'can access from execute')

  t.deepEqual(c.execute('return foo'), c.execute('return globals.foo'), 'can acess from execute directly (scoped within globals)')

  t.deepEqual(c.execute('return foo', {foo: pack('bar')}), {errors: null, output: pack('bar')}, 'inputs (locals) mask globals')

  t.deepEqual(c.execute('return foo'), c.execute('return globals.foo'), 'inputs only mask globals per call')

  t.end()
})

test('JsContext will transform code to ES2015(ES6)', function (t) {
  let c = new JsContext({
    transform: true
  })

  t.deepEqual(c.execute('return Math.max(...[1,3,2])'), {errors: null, output: pack(3)})

  t.end()
})

if (typeof window !== 'undefined') {
  test('JsContext can dynamically require NPM modules', t => {
    let c = new JsContext()

    t.deepEqual(c.execute('let isNumber = require("is-number")\nisNumber(1)'), {errors: null, output: pack(true)})

    t.end()
  })
}
