import { execute, parseCodeChunk } from './interpreter'
import { codeChunk, CodeError, codeExpression } from './types'

describe('Code execution', () => {
  test('it executes CodeExpressions and captures the result. Errors are captured and subsequent Code Expressions are still parsed', () => {
    const exprs = [
      codeExpression('2 + 5'),
      codeExpression('bad syntax'),
      codeExpression('a * 2')
    ]
    execute(exprs, { a: 100 })
    expect(exprs[0].output).toEqual(7)

    expect(exprs[1].errors).toHaveLength(1)
    expect((exprs[1].errors as CodeError[])[0].kind).toEqual('SyntaxError')

    expect(exprs[2].output).toEqual(200)
  })

  test('it executes CodeChunks with multiple statements, results flow through and errors are captured', () => {
    const chunks = [
      codeChunk(
        'function doubler(d) { return d * 2 }\nlet a = 5\ndoubler(a * b)\nc\nconsole.log(a)'
      ),
      codeChunk("def: a_python_func():\n    print('what is this, python?')\n"),
      codeChunk('console.log(100)\nbadFunction(d)')
    ]
    const processedChunks = chunks.map(cc => {
      return { codeChunk: cc, parseResult: parseCodeChunk(cc) }
    })

    execute(processedChunks, { b: 20 })

    const outputs1 = chunks[0].outputs as (string | number)[]
    expect(outputs1).toHaveLength(2)
    expect(outputs1[0]).toEqual(200)
    expect(outputs1[1]).toEqual('5')

    // Don't know how long it should take, > 0 though
    expect(chunks[0].duration).toBeGreaterThan(0)

    expect(chunks[1].outputs).toBeUndefined()

    const outputs2 = chunks[2].outputs as (string | number)[]

    expect(outputs2).toHaveLength(1)
    expect(outputs2[0]).toEqual('100')

    expect(chunks[2].duration).toBeGreaterThan(0)

    const errors2 = chunks[2].errors as CodeError[]
    expect(errors2).toHaveLength(1)
    expect(errors2[0].kind).toEqual('ReferenceError')
    expect(errors2[0].message).toEqual('badFunction is not defined')
    expect(errors2[0].trace).toBeDefined()
  })
})
