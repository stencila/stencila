import { codeChunk, Function, Parameter } from './types'
import { parseCodeChunk } from './executor'

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
