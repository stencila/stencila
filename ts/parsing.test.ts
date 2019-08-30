import {
  codeChunk,
  CodeExpression,
  IntegerSchema,
  Function,
  Parameter,
  parameter,
  constantSchema,
  enumSchema,
  booleanSchema,
  integerSchema,
  numberSchema,
  stringSchema,
  arraySchema,
  tupleSchema
} from './types'
import {
  CodeChunkExecution,
  ExecutableCode,
  parseItem,
  parseCodeChunk,
  decodeParameters
} from './interpreter'

describe('Code chunk parsing', () => {
  test('it parses variable declarations', () => {
    const result = parseCodeChunk(
      codeChunk('const a = 5\nlet b = 6\nvar c = 7')
    )
    expect(result.declares).toHaveLength(3)
    expect(result.declares[0].name).toEqual('a')
    expect(result.declares[1].name).toEqual('b')
    expect(result.declares[2].name).toEqual('c')
  })

  test('it parses named function declarations', () => {
    const result = parseCodeChunk(
      codeChunk('function testFunc(a, b, ...args) {\n    return 7\n}')
    )

    expect(result.declares).toHaveLength(1)
    expect(result.declares[0].type).toEqual('Function')
    expect(result.declares[0].name).toEqual('testFunc')

    const parameters = (result.declares[0] as Function)
      .parameters as Parameter[]

    expect(parameters).toHaveLength(3)
    expect(parameters[0].name).toEqual('a')
    expect(parameters[0].required).toEqual(true)
    expect(parameters[0].extends).toEqual(false)
    expect(parameters[0].repeats).toEqual(false)

    expect(parameters[1].name).toEqual('b')
    expect(parameters[1].required).toEqual(true)
    expect(parameters[1].extends).toEqual(false)
    expect(parameters[1].repeats).toEqual(false)

    expect(parameters[2].name).toEqual('args')
    expect(parameters[2].required).toEqual(false)
    expect(parameters[2].extends).toEqual(false)
    expect(parameters[2].repeats).toEqual(true)
  })

  test('it parses assigned function declarations', () => {
    const result = parseCodeChunk(
      codeChunk('const otherFunc = function(d, e, ...zargs) {\n    return 9\n}')
    )

    expect(result.declares).toHaveLength(1)
    expect(result.declares[0].type).toEqual('Function')
    expect(result.declares[0].name).toEqual('otherFunc')

    const parameters = (result.declares[0] as Function)
      .parameters as Parameter[]

    expect(parameters).toHaveLength(3)
    expect(parameters[0].name).toEqual('d')
    expect(parameters[0].required).toEqual(true)
    expect(parameters[0].extends).toEqual(false)
    expect(parameters[0].repeats).toEqual(false)

    expect(parameters[1].name).toEqual('e')
    expect(parameters[1].required).toEqual(true)
    expect(parameters[1].extends).toEqual(false)
    expect(parameters[1].repeats).toEqual(false)

    expect(parameters[2].name).toEqual('zargs')
    expect(parameters[2].required).toEqual(false)
    expect(parameters[2].extends).toEqual(false)
    expect(parameters[2].repeats).toEqual(true)
  })

  test('it parses variables assigned and used in assignment', () => {
    const result = parseCodeChunk(codeChunk('a = 5 / b\nc = d / e'))
    expect(result.assigns.sort()).toEqual(['a', 'c'])
    expect(result.uses.sort()).toEqual(['b', 'd', 'e'])
  })

  test('it parses variables used in function calls', () => {
    const result = parseCodeChunk(codeChunk('callFunc(a, b, c)'))
    expect(result.uses.sort()).toEqual(['a', 'b', 'c'])
  })

  test('it parses variables that are altered (but not assigned/declared)', () => {
    const result = parseCodeChunk(
      codeChunk('a[2][3] = 1\nb.c = 4\nd++\n++e\n--f')
    )
    expect(result.alters.sort()).toEqual(['a', 'b', 'd', 'e', 'f'])
  })

  test('it parses complex structures', () => {
    const result = parseCodeChunk(codeChunk('a = [b, c, {d, e: f, g: h[i]}]'))
    expect(result.uses.sort()).toEqual(['b', 'c', 'd', 'e', 'f', 'g', 'h', 'i'])
    expect(result.assigns).toEqual(['a'])

    const result2 = parseCodeChunk(codeChunk('a = [b.c.d, e]'))
    expect(result2.uses.sort()).toEqual(['b', 'e'])
    expect(result2.assigns).toEqual(['a'])
  })

  test('it parses if statements', () => {
    const result = parseCodeChunk(
      codeChunk(
        'if ( a > b || c < d && e === f ) {\n    callFunc(g)\n} else if ( h === j ){\n    k = l\n} else {\n    m = n / o\n} '
      )
    )
    expect(result.uses.sort()).toEqual([
      'a',
      'b',
      'c',
      'd',
      'e',
      'f',
      'g',
      'h',
      'j',
      'l',
      'n',
      'o'
    ])
    expect(result.assigns.sort()).toEqual(['k', 'm'])
  })

  test('it parses while loops', () => {
    const result = parseCodeChunk(
      codeChunk('while ( a > b ) {\n    c = d++\n}')
    )
    expect(result.uses.sort()).toEqual(['a', 'b'])
    expect(result.assigns).toEqual(['c'])
    expect(result.alters).toEqual(['d'])
  })

  test('it parses do while loops', () => {
    const result = parseCodeChunk(
      codeChunk('do {\n    a = b + 1\n} while (c > a)')
    )
    expect(result.uses.sort()).toEqual(['b', 'c'])
    expect(result.assigns).toEqual(['a'])
  })

  test('it parses for loops', () => {
    const result = parseCodeChunk(codeChunk('for(let i = 0; ++i; i < j) a = i'))

    // for loops are a weird case – don't have the i as a declaration
    expect(result.declares).toHaveLength(0)
    expect(result.assigns).toEqual(['a'])
  })

  test('it parses for in loops', () => {
    const result = parseCodeChunk(codeChunk('for(let c in d ) { e ++ }'))
    expect(result.uses).toEqual(['d'])
    expect(result.assigns).toHaveLength(0)
    expect(result.alters).toEqual(['e'])
  })

  test('it parses for of loops', () => {
    const result = parseCodeChunk(codeChunk('for(let c of d ) { e ++ }'))
    expect(result.uses).toEqual(['d'])
    expect(result.assigns).toHaveLength(0)
    expect(result.alters).toEqual(['e'])
  })

  test('it parses switch statements', () => {
    const result = parseCodeChunk(
      codeChunk(
        'switch (a.b) {\ncase c:\nrunFunc(d)\nbreak\ncase 2:\ne = 1\nbreak\ncase 3:\ncase 4:\nbreak\ndefault:\nf = 2}'
      )
    )
    expect(result.uses.sort()).toEqual(['a', 'c', 'd'])
    expect(result.assigns.sort()).toEqual(['e', 'f'])
  })

  test('it parses all types of imports', () => {
    const result = parseCodeChunk(
      codeChunk(
        "import defaultExport from 'module-name'\n" +
          "import * as name from 'module-name2'\n" +
          "import { export1 } from 'module-name3'\n" +
          "import { export1 as alias1 } from 'module-name4'\n" +
          "import { export1 , export2 } from 'module-name5'"
      )
    )

    expect(result.imports).toEqual([
      'module-name',
      'module-name2',
      'module-name3',
      'module-name4',
      'module-name5'
    ])
  })

  test('it parses file reads', () => {
    const result = parseCodeChunk(
      codeChunk(
        "fs.readFileSync('read1')\nreadFileSync('read2')\nfs.readFile('read3')\nreadFile('read4')\nfunction testFunc() {\nopen('read5', 'r')\nopen('read6', 'w+')\nopen('write1', 'w')\nopen('append1', 'a') }"
      )
    )

    expect(result.reads).toEqual([
      'read1',
      'read2',
      'read3',
      'read4',
      'read5',
      'read6'
    ])
  })

  test('it stores errors that happen when parsing the code', () => {
    const result = parseCodeChunk(codeChunk('this is invalid code'))

    expect(result.errors[0].kind).toEqual('SyntaxError')
    expect(result.errors[0].message).toContain('Unexpected token')
    expect(result.errors[0].trace).toBeDefined()
  })
})

describe('Parsing information from an Article', () => {
  const chunk1Text = 'var a = 4'
  const chunk2Text = 'invalid javascript code'
  const chunk3Text = 'function somefunc(bad_param){\n    return bad_param\n}'

  const expr1Text = 'a * parameter_one'
  const expr2Text = 'more invalid javascript code'

  const article = {
    type: 'Article',
    title: 'Upcoming Temperatures',
    authors: [],
    content: [
      {
        type: 'Parameter',
        name: 'parameter_one',
        schema: {
          type: 'IntegerSchema'
        }
      },
      {
        type: 'Heading',
        depth: 1,
        content: ['A Heading']
      },
      {
        type: 'CodeChunk',
        text: "def a():\n    return 'I am python!'",
        language: 'notjavascript'
      },
      {
        type: 'CodeChunk',
        text: chunk1Text,
        language: 'javascript'
      },
      {
        type: 'CodeChunk',
        text: chunk2Text,
        language: 'javascript'
      },
      {
        type: 'CodeChunk',
        text: chunk3Text,
        language: 'javascript',
        declares: [
          {
            type: 'Function',
            name: 'somefunc',
            parameters: [
              {
                type: 'Parameter',
                name: 'bad_param',
                required: true
              }
            ]
          }
        ]
      },
      {
        type: 'CodeExpression',
        text: 'invalid code',
        language: 'notjavascript'
      },
      {
        type: 'CodeExpression',
        text: expr1Text,
        language: 'javascript'
      },
      {
        type: 'CodeExpression',
        text: expr2Text,
        language: 'javascript'
      }
    ]
  }

  const parameters: Parameter[] = []
  const code: ExecutableCode[] = []

  parseItem(article, parameters, code)

  test('it parses only Javascript codes and code chunks', () => {
    expect(code).toHaveLength(5)
    expect((code[0] as CodeChunkExecution).codeChunk.text).toEqual(chunk1Text)
    expect((code[1] as CodeChunkExecution).codeChunk.text).toEqual(chunk2Text)
    expect((code[2] as CodeChunkExecution).codeChunk.text).toEqual(chunk3Text)

    expect((code[3] as CodeExpression).text).toEqual(expr1Text)
    expect((code[4] as CodeExpression).text).toEqual(expr2Text)

    expect((code[0] as CodeChunkExecution).codeChunk.language).toEqual(
      'javascript'
    )
    expect((code[1] as CodeChunkExecution).codeChunk.language).toEqual(
      'javascript'
    )
    expect((code[2] as CodeChunkExecution).codeChunk.language).toEqual(
      'javascript'
    )

    expect((code[3] as CodeExpression).language).toEqual('javascript')
    expect((code[4] as CodeExpression).language).toEqual('javascript')
  })

  test('it only parses Parameters that are not inside a Function', () => {
    expect(parameters).toHaveLength(1)
    expect(parameters[0].name).toEqual('parameter_one')
    expect((parameters[0].schema as IntegerSchema).type).toEqual(
      'IntegerSchema'
    )
  })
})

describe('CLI parameter parsing', () => {
  test('it parses ConstantSchema parameters', () => {
    const parameters = [
      parameter('const', {
        schema: constantSchema({ value: 'abc123' })
      })
    ]

    const d1 = decodeParameters(parameters, {})
    expect(d1.const).toEqual('abc123')

    const d2 = decodeParameters(parameters, { const: 'def456' })
    expect(d2.const).toEqual('abc123')
  })

  test('it parses EnumSchema parameters', () => {
    const parameters = [
      parameter('enum1', { schema: enumSchema({ values: ['a', 'b', 'c'] }) }),
      parameter('enum2', {
        schema: enumSchema({ values: ['d', 'e', 'f'] }),
        default: 'e',
        required: false
      })
    ]

    const d1 = decodeParameters(parameters, { enum1: 'b' })
    expect(d1.enum1).toEqual('b')
    expect(d1.enum2).toEqual('e')

    expect(() => {
      decodeParameters(parameters, { enum1: 'g' })
    }).toThrowError('g not found in enum values for enum1')
  })

  test('it parses BooleanSchema parameters', () => {
    const parameters = [
      parameter('bool1', { schema: booleanSchema() }),
      parameter('bool2', {
        schema: booleanSchema(),
        default: true,
        required: false
      })
    ]

    const d1 = decodeParameters(parameters, { bool1: 'true' })
    expect(d1.bool1).toEqual(true)
    expect(d1.bool2).toEqual(true)

    expect(decodeParameters(parameters, { bool1: 't' }).bool1).toEqual(true)
    expect(decodeParameters(parameters, { bool1: 'TRUe' }).bool1).toEqual(true)
    expect(decodeParameters(parameters, { bool1: 'Yes' }).bool1).toEqual(true)
    expect(decodeParameters(parameters, { bool1: '1' }).bool1).toEqual(true)

    expect(decodeParameters(parameters, { bool1: 'f' }).bool1).toEqual(false)
    expect(decodeParameters(parameters, { bool1: 'false' }).bool1).toEqual(
      false
    )
    expect(decodeParameters(parameters, { bool1: 'NO' }).bool1).toEqual(false)
    expect(decodeParameters(parameters, { bool1: '0' }).bool1).toEqual(false)
  })

  test('it parses IntegerSchema parameters', () => {
    const parameters = [
      parameter('int1', { schema: integerSchema() }),
      parameter('int2', {
        schema: integerSchema(),
        default: 1000,
        required: false
      })
    ]

    const d1 = decodeParameters(parameters, { int1: '50' })
    expect(d1.int1).toEqual(50)
    expect(d1.int2).toEqual(1000)
  })

  test('it parses NumberSchema parameters', () => {
    const parameters = [
      parameter('num1', { schema: numberSchema() }),
      parameter('num2', {
        schema: numberSchema(),
        default: 3.1416,
        required: false
      })
    ]

    const d1 = decodeParameters(parameters, { num1: '2.7181' })
    expect(d1.num1).toEqual(2.7181)
    expect(d1.num2).toEqual(3.1416)
  })

  test('it parses StringSchema parameters', () => {
    const parameters = [
      parameter('string1', { schema: stringSchema() }),
      parameter('string2', {
        schema: stringSchema(),
        default: 'def345',
        required: false
      }),
      parameter('val1') // with no schema, just pass the value through
    ]

    const d1 = decodeParameters(parameters, {
      string1: '2.7181',
      val1: 'some string'
    })
    expect(d1.string1).toEqual('2.7181')
    expect(d1.string2).toEqual('def345')
    expect(d1.val1).toEqual('some string')
  })

  test('it parses ArraySchema parameters', () => {
    const parameters = [
      parameter('array1', { schema: arraySchema() }),
      parameter('array2', {
        schema: arraySchema(),
        default: [3, 4, 5],
        required: false
      })
    ]

    const d1 = decodeParameters(parameters, { array1: '[1, 2, "a"]' })
    expect(d1.array1).toEqual([1, 2, 'a'])
    expect(d1.array2).toEqual([3, 4, 5])
  })

  test('it parses TupleSchema parameters', () => {
    const parameters = [
      parameter('tuple1', { schema: tupleSchema() }),
      parameter('tuple2', {
        schema: tupleSchema(),
        default: [1, true],
        required: false
      })
    ]

    const d1 = decodeParameters(parameters, { tuple1: '[0, "55"]' })
    expect(d1.tuple1).toEqual([0, '55'])
    expect(d1.tuple2).toEqual([1, true])
  })

  test('it throws an error if required parameters are missing', () => {
    const parameters = [parameter('p', { required: true })]

    expect(() => {
      decodeParameters(parameters, {})
    }).toThrowError('No value or default found for parameter p')
  })
})
