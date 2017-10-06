import {type, pack} from '../../src/value'
import JsContext from '../../src/contexts/JsContext'

import test from 'tape'

test('JsContext', t => {
  let c = new JsContext()

  t.ok(c instanceof JsContext)
  t.equal(c.constructor.name, 'JsContext')

  t.end()
})

test('JsContext.supportedLanguages', t => {
  let c = new JsContext()

  c.supportedLanguages().then(languages => {
    t.deepEqual(languages, ['js'])
    t.end()
  })
})

test('JsContext.supportsLanguage', t => {
  let c = new JsContext()

  t.plan(2)

  c.supportsLanguage('js').then(result => {
    t.equal(result, true)
  })

  c.supportsLanguage('py').then(result => {
    t.equal(result, false)
  })
})

test('JsContext.analyseCode', t => {
  let c = new JsContext()
  t.plan(10)

  c.analyseCode('Math.pi').then(result => t.deepEqual(result, {
    inputs: [],
    output: null,
    value: 'Math.pi',
    errors: []
  }))

  c.analyseCode('foo').then(result => t.deepEqual(result, {
    inputs: ['foo'],
    output: 'foo',
    value: 'foo',
    errors: []
  }))

  c.analyseCode('let foo\nfoo').then(result => t.deepEqual(result, {
    inputs: [],
    output: 'foo',
    value: 'foo',
    errors: []
  }))


  c.analyseCode('let foo\nfoo * 3').then(result => t.deepEqual(result, {
    inputs: [],
    output: null,
    value: 'foo * 3',
    errors: []
  }))

  c.analyseCode('let foo').then(result => t.deepEqual(result, {
    inputs: [],
    output: 'foo',
    value: 'foo',
    errors: []
  }))

  // Last statement is a declaration (first identifier used)
  c.analyseCode('foo\nbar\nlet baz, urg\n\n').then(result => t.deepEqual(result, {
    inputs: ['foo','bar'],
    output: 'baz',
    value: 'baz',
    errors: []
  }))

  // Last statement is not a declaration or identifier
  c.analyseCode('let foo\n{bar\nlet baz}').then(result => t.deepEqual(result, {
    inputs: ['bar'],
    output: null,
    value: null,
    errors: []
  }))

  // Last statement is not a declaration or identifier
  c.analyseCode('let foo\nbar\nlet baz\ntrue').then(result => t.deepEqual(result, {
    inputs: ['bar'],
    output: null,
    value: 'true',
    errors: []
  }))

  // Variable declaration after usage (this will be a runtime error but this tests static analysis of code regardless)
  c.analyseCode('foo\nlet foo\n').then(result => t.deepEqual(result, {
    inputs: ['foo'],
    output: 'foo',
    value: 'foo',
    errors: []
  }))

  // Syntax error
  c.analyseCode('for(){').then(result => t.deepEqual(result, {
    inputs: [],
    output: null,
    value: null,
    errors: [ { line: 1, column: 4, message: 'SyntaxError: Unexpected token (1:4)'} ]
  }))
})

test('JsContext.executeCode no value', t => {
  let c = new JsContext()
  t.plan(2)

  c.executeCode('\n').then(result => {
    t.deepEqual(result.value, null, 'nothing returned when empty')
  })

  c.executeCode('if(true){\n  let x = 4\n}\n').then(result => {
    t.deepEqual(result, {
      inputs: [],
      output: null,
      value: null,
      errors: []
    })
  })
})

test('JsContext.executeCode with no inputs, no output, no errors', t => {
  let c = new JsContext()
  t.plan(3)

  c.executeCode('1.1 * 2').then(result => {
    t.deepEqual(result, {
      inputs: [],
      output: null,
      value: { type: 'number', data: 2.2 },
      errors: []
    })
  })

  c.executeCode('let x = 3\nMath.sqrt(x*3)').then(result => {
    t.deepEqual(result, {
      inputs: [],
      output: null,
      value: { type: 'integer', data: 3 },
      errors: []
    })
  })

  c.executeCode('// Multiple lines and comments\nlet x = {}\nObject.assign(x, {a:1})\n').then(result => {
    t.deepEqual(result, {
      inputs: [],
      output: null,
      value: { type: 'object', data: { a: 1 } },
      errors: []
    })
  })
})

test('JsContext.executeCode with inputs, outputs, no errors', t => {
  let c = new JsContext()
  t.plan(2)

  c.executeCode('let b = a*6', {
    a: {type: 'integer', data: 6}
  }).then(result => {
    t.deepEqual(result, {
      inputs: ['a'],
      output: 'b',
      value: { type: 'integer', data: 36 },
      errors: []
    })
  })

  c.executeCode('let c = a*b[1]\nc', {
    a: {type: 'integer', data: 6},
    b: {type: 'array[number]', data: [1, 2, 3]}
  }).then(result => {
    t.deepEqual(result, {
      inputs: ['a', 'b'],
      output: 'c',
      value: { type: 'integer', data: 12 },
      errors: []
    })
  })
})

test('JsContext.executeCode value is multiline', t => {
  let c = new JsContext()
  t.plan(1)

  c.executeCode(`let x = {
    a: 1, 
    b: "foo"
  }`).then(result => {
    t.deepEqual(result, {
      inputs: [],
      output: 'x',
      value: { type: 'object', data: { a: 1, b: 'foo'} },
      errors: []
    })
  })
})

test('JsContext.executeCode with errors', t => {
  let c = new JsContext()
  t.plan(3)

  c.executeCode('foo').then(result => {
    t.deepEqual(result.errors, [{line: 1, column: 1, message: 'ReferenceError: foo is not defined'}])
  })
  c.executeCode('1\n2\n foo\n4').then(result => {
    t.deepEqual(result.errors, [{line: 3, column: 2, message: 'ReferenceError: foo is not defined'}])
  })
  c.executeCode(' <>').then(result => {
    t.deepEqual(result.errors, [{line: 1, column: 1, message: 'SyntaxError: Unexpected token (1:1)'}])
  })
})

test('JsContext.executeCode with global variables', t => {
  let c = new JsContext()
  t.plan(3)

  c.executeCode('foo = "bar"')

  c.executeCode('foo').then(result => {
    t.deepEqual(result.value, {type: 'string', data: 'bar'}, 'can get global variable')
  })

  c.executeCode('foo + "t_simpson"').then(result => {
    t.deepEqual(result.value, {type: 'string', data: 'bart_simpson'}, 'can get global variable expression')
  })

  c.executeCode('foo = 42')

  c.executeCode('foo').then(result => {
    t.deepEqual(result.value, {type: 'integer', data: 42}, 'can change global variable')
  })
})

test.skip('JsContext.hasFunction', t => {
  let c = new JsContext()

  t.ok(c.hasFunction('type'))
  t.notOk(c.hasFunction('this_is_not_a_registered_function'))
  t.end()
})

test.skip('JsContext.callFunction without function name', t => {
  let c = new JsContext()
  t.plan(1)

  t.throws(() => {
    c.callFunction()
  })
})

test.skip('JsContext.callFunction with no inputs', t => {
  let c = new JsContext()
  t.plan(1)

  c.callFunction('type').then(result => {
    t.deepEqual(result, {output: pack('unknown'), errors: null})
  })
})

test.skip('JsContext.callFunction with inputs and output', t => {
  let c = new JsContext()
  t.plan(1)

  c.callFunction('type', [pack(1)]).then(result => {
    t.deepEqual(result, {output: pack('integer'), errors: null})
  })
})

test.skip('JsContext.callFunction with named arguments', t => {
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

test.skip('JsContext.callFunction with error', t => {
  let c = new JsContext()
  t.plan(1)
  c._functions['foo'] = () => {
    throw new Error('nope')
  }
  c.callFunction('foo').then(result => {
    t.deepEqual(result, {errors: [ { column: 0, line: 0, message: 'Error: nope' } ], output: null})
  })
})
