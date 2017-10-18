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
    messages: []
  }))

  c.analyseCode('foo').then(result => t.deepEqual(result, {
    inputs: ['foo'],
    output: 'foo',
    messages: []
  }))

  c.analyseCode('let foo\nfoo').then(result => t.deepEqual(result, {
    inputs: [],
    output: 'foo',
    messages: []
  }))


  c.analyseCode('let foo\nfoo * 3').then(result => t.deepEqual(result, {
    inputs: [],
    output: null,
    messages: []
  }))

  c.analyseCode('let foo').then(result => t.deepEqual(result, {
    inputs: [],
    output: 'foo',
    messages: []
  }))

  // Last statement is a declaration (first identifier used)
  c.analyseCode('foo\nbar\nlet baz, urg\n\n').then(result => t.deepEqual(result, {
    inputs: ['foo','bar'],
    output: 'baz',
    messages: []
  }))

  // Last statement is not a declaration or identifier
  c.analyseCode('let foo\n{bar\nlet baz}').then(result => t.deepEqual(result, {
    inputs: ['bar'],
    output: null,
    messages: []
  }))

  // Last statement is not a declaration or identifier
  c.analyseCode('let foo\nbar\nlet baz\ntrue').then(result => t.deepEqual(result, {
    inputs: ['bar'],
    output: null,
    messages: []
  }))

  // Variable declaration after usage (this will be a runtime error but this tests static analysis of code regardless)
  c.analyseCode('foo\nlet foo\n').then(result => t.deepEqual(result, {
    inputs: ['foo'],
    output: 'foo',
    messages: []
  }))

  // Syntax error
  c.analyseCode('for(){').then(result => t.deepEqual(result, {
    inputs: [],
    output: null,
    messages: [ { line: 1, column: 4, type: 'error', message: 'SyntaxError: Unexpected token (1:4)'} ]
  }))
})

test('JsContext.analyseCode expression only', t => {
  let c = new JsContext()
  t.plan(6)

  c.analyseCode('42', true).then(result => t.deepEqual(result, {
    inputs: [],
    output: null,
    messages: []
  }))

  c.analyseCode('x * 3', true).then(result => t.deepEqual(result, {
    inputs: ['x'],
    output: null,
    messages: []
  }))

  c.analyseCode('let y = x * 3', true).then(result => t.deepEqual(result, {
    inputs: [],
    output: null,
    messages: [{ line: 0, column: 0, type: 'error', message: 'Error: Code is not a single, simple expression' }]
  }))

  c.analyseCode('y = x * 3', true).then(result => t.equal(
    result.messages[0].message,
    'Error: Code is not a single, simple expression'
  ))

  c.analyseCode('x++', true).then(result => t.equal(
    result.messages[0].message,
    'Error: Code is not a single, simple expression'
  ))

  c.analyseCode('y--', true).then(result => t.equal(
    result.messages[0].message,
    'Error: Code is not a single, simple expression'
  ))
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
      messages: [],
      streams: null
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
      messages: [],
      streams: null
    })
  })

  c.executeCode('let x = 3\nMath.sqrt(x*3)').then(result => {
    t.deepEqual(result, {
      inputs: [],
      output: null,
      value: { type: 'integer', data: 3 },
      messages: [],
      streams: null
    })
  })

  c.executeCode('// Multiple lines and comments\nlet x = {}\nObject.assign(x, {a:1})\n').then(result => {
    t.deepEqual(result, {
      inputs: [],
      output: null,
      value: { type: 'object', data: { a: 1 } },
      messages: [],
      streams: null
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
      messages: [],
      streams: null
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
      messages: [],
      streams: null
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
      messages: [],
      streams: null
    })
  })
})

test('JsContext.executeCode with errors', t => {
  let c = new JsContext()
  t.plan(3)

  c.executeCode('foo').then(result => {
    t.deepEqual(result.messages, [
      { line: 0, column: 0, type: 'warn', message: 'Input variable "foo" is not managed' },
      { line: 1, column: 1, type: 'error', message: 'ReferenceError: foo is not defined' }
    ])
  })
  c.executeCode('1\n2\n foo\n4').then(result => {
    t.deepEqual(result.messages, [
      { line: 0, column: 0, type: 'warn', message: 'Input variable "foo" is not managed' },
      { line: 3, column: 2, type: 'error', message: 'ReferenceError: foo is not defined' }
    ])
  })
  c.executeCode(' <>').then(result => {
    t.deepEqual(result.messages, [
      { line: 1, column: 1, type: 'error', message: 'SyntaxError: Unexpected token (1:1)' }
    ])
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

test('JsContext.executeCode with console output', t => {
  let c = new JsContext()
  t.plan(3)

  c.executeCode('console.log("Hello!")').then(result => {
    t.equal(result.streams.stdout, "Hello!")
  })

  c.executeCode('console.warn("Warning")').then(result => {
    t.equal(result.streams.stdout, "Warning")
  })

  c.executeCode('console.error("Errrrr!")').then(result => {
    t.equal(result.streams.stderr, "Errrrr!")
  })
})

test('JsContext.hasFunction', t => {
  let c = new JsContext()
  t.plan(2)

  c.hasFunction('core', 'type').then(result => {
    t.equal(result, true)
  })

  c.hasFunction('foo', 'this_is_not_a_registered_function').then(result => {
    t.equal(result, false)
  })
})

test('JsContext.callFunction', t => {
  let c = new JsContext()
  t.plan(3)

  t.throws(() => {
    c.callFunction()
  })

  c.callFunction('core', 'type', [{type: 'integer', data: 42}]).then(result => {
    t.deepEqual(result.value, {type: 'string', data: 'integer'})
  })

  c._libs['foo'] = {
    bar: function () {
      throw new Error('nope')
    }
  }
  c.callFunction('foo', 'bar').then(result => {
    t.deepEqual(result.messages, [ { column: 0, line: 0, type: 'error', message: 'Error: nope' } ])
  })
})
