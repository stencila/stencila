import getStdin from 'get-stdin'
import minimist from 'minimist'
import { parseScript } from 'meriyah'
import { generate } from 'astring'

import fs from 'fs'
import {
  Article,
  CodeChunk,
  CodeExpression,
  EnumSchema,
  Parameter
} from './types'
import { isA } from './util'

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main()

type ExecutableCode = CodeChunk | CodeExpression

interface StringDict {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any
}

interface CliArgs {
  inputFile: string

  outputFile: string

  parameterValues: StringDict
}

/**
 * Process argv to get the source and destination for the document being executed
 */
function getCliArgs(): CliArgs {
  const { _, ...params } = minimist(process.argv.slice(2), {})

  let inFile = '-'
  let outFile = '-'

  if (_.length === 0) {
    // read from stdin and output to stdout
  }

  if (_.length >= 1) {
    // read from provided file arg which might be -, output to stdout
    inFile = _[0]

    if (_.length >= 2) {
      // read from file, out to file. but they might be -
      outFile = _[1]
    }
  }

  return {
    inputFile: inFile,
    outputFile: outFile,
    parameterValues: params
  }
}

/**
 * Execute each item in `code`, making each `parameter` available in the execution context scope.
 */
function execute(code: ExecutableCode[], parameterValues: StringDict): void {
  Object.entries(parameterValues).forEach(([key, value]) => {
    // Add each parameter into the global scope
    // @ts-ignore
    global[key] = value
  })

  code.forEach(c => {
    if (isA('CodeChunk', c)) {
      executeCodeChunk(c)
    } else if (isA('CodeExpression', c)) {
      executeCodeExpression(c)
    }
  })
}

/**
 * Traverse a `Node` hierarchy and push any `Parmamater` that is found onto the `paramaters` array,
 * and any executable code block onto the `code` array.
 */
function parseItem(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  item: any,
  parameters: Parameter[],
  code: (CodeChunk | CodeExpression)[]
): void {
  if (isA('Entity', item) || item instanceof Object) {
    if (isA('Parameter', item)) {
      parameters.push(item)
    } else if (
      (isA('CodeChunk', item) || isA('CodeExpression', item)) &&
      item.language === 'javascript'
    ) {
      code.push(item)
    }

    Object.entries(item).forEach(([, i]) => {
      parseItem(i, parameters, code)
    })
  } else if (Array.isArray(item)) {
    item.forEach(i => parseItem(i, parameters, code))
  }
}

/**
 * Execute a `CodeExpression` (which should be a single expression) and set the result onto its `output` property.
 *
 * Uses `eval`, so should only be called with trusted code.
 */
function executeCodeExpression(code: CodeExpression): void {
  // eslint-disable-next-line no-eval
  code.output = eval(code.text)
}

/**
 * Execute a `CodeChunk` (which might be multiple lines) and set any return values or console output as items in an
 * array on it `outputs` property.
 *
 * Uses `eval`, so should only be called with trusted code.
 */
function executeCodeChunk(code: CodeChunk): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const outputs: any[] = []

  const ast = parseScript(code.text)

  ast.body.forEach(statement => {
    /*
    Convert `const` or `let` global declarations to `var`, otherwise they are lost in subsequent eval() calls due to
    scope changes
     */
    if (statement.type === 'VariableDeclaration' && statement.kind !== 'var')
      statement.kind = 'var'

    // @ts-ignore
    const generatedCode = generate(statement)

    let loggedData = ''

    const oldCl = console.log

    console.log = (s: string) => {
      loggedData += s
    }

    // @ts-ignore
    // eslint-disable-next-line no-eval
    const res = (1, eval)(generatedCode)

    console.log = oldCl

    outputs.push(res)

    if (loggedData.length > 0) {
      outputs.push(loggedData)
    }
  })

  code.outputs = outputs.filter(o => o !== undefined)
}

/**
 * Decode a parameter (e.g. read from the command line, a string) into a value based on the `schema` of the `parameter`.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function decodeParameter(parameter: Parameter, value?: string): any {
  if (isA('ConstantSchema', parameter.schema)) return parameter.schema.value

  if (value === undefined) {
    if (parameter.default === undefined)
      throw new Error(
        `No value or default found for parameter ${parameter.name}`
      )

    return parameter.default
  }

  if (parameter.schema === undefined) return value

  switch (parameter.schema.type) {
    case 'ArraySchema':
    case 'TupleSchema':
      return JSON.parse(value)
    case 'BooleanSchema':
      const b = value.toLowerCase()
      return b === 'true' || b === 't' || b === 'yes' || b === '1'
    case 'EnumSchema':
      const es = parameter.schema as EnumSchema
      if (es.values !== undefined && es.values.indexOf(value) === -1)
        throw new Error(
          `${value} not found in enum values for ${parameter.name}`
        )
      return value
    case 'IntegerSchema':
      const intValue = parseInt(value)
      if (isNaN(intValue)) throw new Error(`$[raw} is not a valid integer`)
      return intValue
    case 'NumberSchema':
      const floatValue = parseFloat(value)
      if (isNaN(floatValue)) throw new Error(`$[raw} is not a valid float`)
      return floatValue
  }

  return value
}

/**
 * Decode all the parameters read from the command line, based on the `Parameter` nodes found in the document.
 */
function decodeParameters(
  parameters: Parameter[],
  values: { [key: string]: string }
): { [key: string]: any } {
  const decodedValues: { [key: string]: any } = {}

  parameters.forEach(p => {
    decodedValues[p.name] = decodeParameter(p, values[p.name])
  })

  return decodedValues
}

/**
 * Read the input document from a file or stdin.
 */
async function readInput(path: string): Promise<string> {
  if (path === '-') {
    return getStdin()
  }

  return fs.readFileSync(path, 'utf8')
}

/**
 * Write the executed document (`Article`) to a file or stdout.
 */
function outputArticle(path: string, output: Article): void {
  const j = JSON.stringify(output)

  if (path === '-') {
    console.log(j)
  } else {
    fs.writeFileSync(path, j)
  }
}

/**
 * Execute a document based on arguments from the command line.
 */
async function main() {
  const cliArgs = getCliArgs()
  const article = JSON.parse(await readInput(cliArgs.inputFile))

  if (!isA('Article', article)) throw TypeError('Not an Article')

  const parameters: Parameter[] = []
  const code: ExecutableCode[] = []

  parseItem(article, parameters, code)

  execute(code, decodeParameters(parameters, cliArgs.parameterValues))

  outputArticle(cliArgs.outputFile, article)
}
