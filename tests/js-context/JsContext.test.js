import {pack} from '../../src/packing'
import JsContext from '../../src/js-context/JsContext'

import test from 'tape'

test('JsContext can be constructed with options', t => {
  let c1 = new JsContext()
  let c2 = new JsContext({
    transform: true
  })

  t.equal(c1.options.transform, typeof window !== 'undefined', 'transform defaults to true in browser, false otherwise')
  t.equal(c2.options.transform, true)

  t.end()
})

test('JsContext.call with no inputs, no errors and no output', t => {
  let c = new JsContext()

  t.deepEqual(c.call('let x = 3\n\n'), {errors: null, output: null}, 'assign')

  t.deepEqual(c.call('// Multiple lines and comments\nlet x = {\na:1\n\n}\n\n'), {errors: null, output: null}, 'assign')

  t.end()
})

test('JsContext.call with no inputs, no errors', t => {
  let c = new JsContext()

  t.deepEqual(c.call('return 42'), {errors: null, output: pack(42)}, 'just an evaluation')
  t.deepEqual(c.call('let x = 3\nreturn x*3'), {errors: null, output: pack(9)}, 'assign and return')
  t.deepEqual(c.call('let x = 3\nx*3\n'), {errors: null, output: null}, 'no return so no output')
  t.end()
})

test('JsContext.call with inputs and outputs but no errors', t => {
  let c = new JsContext()

  t.deepEqual(c.call('return a*6', {a: pack(7)}), {errors: null, output: pack(42)})
  t.deepEqual(c.call('return a*b[1]', {a: pack(17), b: pack([1, 2, 3])}), {errors: null, output: pack(34)})
  t.end()
})

test('JsContext.call output multiline', t => {
  let c = new JsContext()

  t.deepEqual(c.call(`return {
    jermaine: 'Hiphopopotamus',
    brett: 'Rhymnoceros'
  }`, null, {pack: false}), {errors: null, output: { brett: 'Rhymnoceros', jermaine: 'Hiphopopotamus' }})
  t.end()
})

test('JsContext.call with errors', t => {
  let c = new JsContext()

  t.deepEqual(c.call('foo'), {errors: { 1: 'ReferenceError: foo is not defined' }, output: null})
  t.deepEqual(c.call('1\n2\nfoo\n4'), {errors: { 3: 'ReferenceError: foo is not defined' }, output: null})
  t.deepEqual(c.call('<>'), {errors: { 0: 'SyntaxError: Unexpected token <' }, output: null})
  t.end()
})

test('JsContext.run', t => {
  let c = new JsContext()

  c.run('foo = "bar"')
  t.equal(foo, 'bar', 'can set global variable') // eslint-disable-line no-undef

  t.deepEqual(c.run('foo'), {errors: null, output: pack('bar')}, 'can get global variable')
  t.deepEqual(c.run('foo + "t_simpson"'), {errors: null, output: pack('bart_simpson')}, 'can get global variable expression')

  t.deepEqual(c.run('foo\n42\n"lisa"'), {errors: null, output: pack('lisa')}, 'last value is returned')

  t.end()
})

test('JsContext.run with errors', t => {
  let c = new JsContext()

  t.deepEqual(c.run('foogazi'), {errors: { 1: 'ReferenceError: foogazi is not defined' }, output: null})
  t.deepEqual(c.run('<>'), {errors: { 0: 'SyntaxError: Unexpected token <' }, output: null})
  t.end()
})

test('JsContext.depends', t => {
  let c = new JsContext()

  t.deepEqual(c.depends('foo'), ['foo'])
  t.deepEqual(c.depends('let foo\n foo'), [])
  t.deepEqual(c.depends('let foo'), [])
  t.end()
})

test('JsContext will transform code to ES2015(ES6)', t => {
  let c = new JsContext({
    transform: true
  })

  t.deepEqual(c.call('return Math.max(...[1,3,2])'), {errors: null, output: pack(3)})

  t.end()
})

if (typeof window !== 'undefined') {
  test('JsContext can dynamically require NPM modules', t => {
    let c = new JsContext()

    t.deepEqual(c.call('let isNumber = require("is-number")\nisNumber(1)'), {errors: null, output: pack(true)})

    t.end()
  })
}
