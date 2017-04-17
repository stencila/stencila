import {type, pack} from '../../src/value'
import JsContext from '../../src/js-context/JsContext'

import test from 'tape'

test('JsContext', t => {
  let c = new JsContext()

  t.ok(c instanceof JsContext)
  t.equal(c.constructor.name, 'JsContext')

  t.end()
})

test('JsContext.callCode with no inputs, no errors and no output', t => {
  let c = new JsContext()
  t.plan(2)

  c.callCode('let x = 3\n\n').then(result => {
    t.deepEqual(result, {errors: null, output: null}, 'assign')
  })

  c.callCode('// Multiple lines and comments\nlet x = {\na:1\n\n}\n\n').then(result => {
    t.deepEqual(result, {errors: null, output: null}, 'assign')
  })
})

test('JsContext.callCode with no inputs, no errors', t => {
  let c = new JsContext()
  t.plan(3)

  c.callCode('return 42').then(result => {
    t.deepEqual(result, {errors: null, output: pack(42)}, 'just an evaluation')
  })
  c.callCode('let x = 3\nreturn x*3').then(result => {
    t.deepEqual(result, {errors: null, output: pack(9)}, 'assign and return')
  })
  c.callCode('let x = 3\nx*3\n').then(result => {
    t.deepEqual(result, {errors: null, output: null}, 'no return so no output')
  })
})

test('JsContext.callCode with inputs and outputs but no errors', t => {
  let c = new JsContext()
  t.plan(2)

  c.callCode('return a*6', {a: pack(7)}).then(result => {
    t.deepEqual(result, {errors: null, output: pack(42)})
  })
  c.callCode('return a*b[1]', {a: pack(17), b: pack([1, 2, 3])}).then(result => {
    t.deepEqual(result, {errors: null, output: pack(34)})
  })
})

test('JsContext.callCode output multiline', t => {
  let c = new JsContext()
  t.plan(1)

  c.callCode(`return {
    jermaine: 'Hiphopopotamus',
    brett: 'Rhymnoceros'
  }`, {}, {pack: false}).then(result => {
    t.deepEqual(result, {errors: null, output: { brett: 'Rhymnoceros', jermaine: 'Hiphopopotamus' }})
  })
})

test('JsContext.callCode with errors', t => {
  let c = new JsContext()
  t.plan(3)

  c.callCode('foo').then(result => {
    t.deepEqual(result, {errors: [{line: 1, column: 1, message: 'ReferenceError: foo is not defined'}], output: null})
  })
  c.callCode('1\n2\n foo\n4').then(result => {
    t.deepEqual(result, {errors: [{line: 3, column: 2, message: 'ReferenceError: foo is not defined'}], output: null})
  })
  c.callCode('<>').then(result => {
    t.deepEqual(result, {errors: [{line: 0, column: 0, message: 'SyntaxError: Unexpected token <'}], output: null})
  })
})

test('JsContext.runCode', t => {
  let c = new JsContext()
  t.plan(6)

  c.runCode('foo = "bar"')
  t.equal(foo, 'bar', 'can set global variable') // eslint-disable-line no-undef

  c.runCode('foo').then(result => {
    t.deepEqual(result, {errors: null, output: pack('bar')}, 'can get global variable')
  })
  c.runCode('foo + "t_simpson"').then(result => {
    t.deepEqual(result, {errors: null, output: pack('bart_simpson')}, 'can get global variable expression')
  })
  c.runCode('foo\n42\n"lisa"').then(result => {
    t.deepEqual(result, {errors: null, output: pack('lisa')}, 'last value is returned')
  })
  c.runCode('\n').then(result => {
    t.deepEqual(result, {errors: null, output: null}, 'nothing returned when empty')
  })
  c.runCode('let x = 5').then(result => {
    t.deepEqual(result, {errors: null, output: null}, 'nothing returned when last line is statement')
  })
})

test('JsContext.runCode with errors', t => {
  let c = new JsContext()
  t.plan(3)

  c.runCode('foogazi').then(result => {
    t.deepEqual(result, { errors: [ { column: 1, line: 1, message: 'ReferenceError: foogazi is not defined' } ], output: null })
  })
  c.runCode('2*45\nfoogazi').then(result => {
    t.deepEqual(result, { errors: [ { column: 1, line: 2, message: 'ReferenceError: foogazi is not defined' } ], output: null })
  })
  c.runCode('<>').then(result => {
    t.deepEqual(result, { errors: [ { column: 0, line: 0, message: 'SyntaxError: Unexpected token <' } ], output: null })
  })
})

test('JsContext.codeDependencies', t => {
  let c = new JsContext()
  t.plan(3)

  c.codeDependencies('foo').then(result => t.deepEqual(result, ['foo']))
  c.codeDependencies('let foo\n foo').then(result => t.deepEqual(result, []))
  c.codeDependencies('let foo').then(result => t.deepEqual(result, []))
})

test('JsContext.hasFunction', t => {
  let c = new JsContext()

  t.ok(c.hasFunction('type'))
  t.notOk(c.hasFunction('this_is_not_a_registered_function'))
  t.end()
})

test('JsContext.callFunction without function name', t => {
  let c = new JsContext()
  t.plan(1)

  t.throws(() => {
    c.callFunction()
  })
})

test('JsContext.callFunction with no inputs', t => {
  let c = new JsContext()
  t.plan(1)

  c.callFunction('type').then(result => {
    t.deepEqual(result, {output: pack('unknown'), errors: null})
  })
})

test('JsContext.callFunction with inputs and output', t => {
  let c = new JsContext()
  t.plan(1)

  c.callFunction('type', [pack(1)]).then(result => {
    t.deepEqual(result, {output: pack('integer'), errors: null})
  })
})

test('JsContext.callFunction with named arguments', t => {
  let c = new JsContext()
  t.plan(7)

  // TODO this uses a stochatic function for testing! Use a deterministic function with
  // named parameters

  c.callFunction('random_uniform', [10], {}, {pack: false}).then(result => {
    t.equal(type(result.output), 'array')
    t.equal(result.output.length, 10)
  })

  c.callFunction('random_uniform', [], {n: 10}, {pack: false}).then(result => {
    t.equal(type(result.output), 'array')
    t.equal(result.output.length, 10)
  })

  c.callFunction('random_uniform', [], {min: 100, n: 1}, {pack: false}).then(result => {
    t.equal(type(result.output), 'float')
    t.ok(result.output < 100)
  })

  c.callFunction('random_uniform', [], {min: 100, foo: 1}, {pack: false})
    .then(() => {
      t.fail('should not resolve')
    })
    .catch(error => {
      t.equal(error.message, 'Invalid named argument "foo"; valid names are "n", "min", "max"')
    })

})

test('JsContext.callFunction with error', t => {
  let c = new JsContext()
  t.plan(1)
  c._functions['foo'] = () => {
    throw new Error('nope')
  }
  c.callFunction('foo').then(result => {
    t.deepEqual(result, {errors: [ { column: 0, line: 0, message: 'Error: nope' } ], output: null})
  })
})
