import getStdin from 'get-stdin'
import minimist from 'minimist'
import { parse } from 'meriyah'
import { generate } from 'astring'

import fs from 'fs'
import {
  Article,
  CodeChunk,
  codeError,
  CodeError,
  CodeExpression,
  EnumSchema,
  Function,
  function_,
  parameter,
  Parameter,
  variable,
  Variable
} from './types'
import { isA } from './util'
import {
  Node as EtreeNode, // In case of conflict with Stencila Node
  BinaryExpression,
  CallExpression,
  FunctionDeclaration,
  FunctionExpression,
  MemberExpression,
  UpdateExpression,
  VariableDeclaration,
  ExpressionStatement,
  AssignmentExpression,
  ImportDeclaration,
  WhileStatement,
  DoWhileStatement,
  IfStatement,
  Program,
  TryStatement
} from 'meriyah/dist/estree'

// eslint-disable-next-line @typescript-eslint/no-floating-promises
main()

interface StringDict {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any
}

interface CliArgs {
  inputFile: string

  outputFile: string

  parameterValues: StringDict
}

interface CodeChunkExecution {
  codeChunk: CodeChunk
  parseResult: CodeChunkParseResult
}

type ExecutableCode = CodeChunkExecution | CodeExpression

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
    if (isA('CodeExpression', c)) executeCodeExpression(c)
    else executeCodeChunk(c)
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
  code: (CodeChunkExecution | CodeExpression)[],
  functionDepth: number = 0
): void {
  if (isA('Entity', item) || item instanceof Object) {
    if (isA('Function', item)) {
      ++functionDepth
    } else if (isA('Parameter', item) && functionDepth === 0) {
      parameters.push(item)
    } else if (
      (isA('CodeChunk', item) || isA('CodeExpression', item)) &&
      item.language === 'javascript'
    ) {
      if (isA('CodeChunk', item)) {
        const parseResult = parseCodeChunk(item)
        parseResult.finalize()

        item.imports = parseResult.imports
        item.declares = parseResult.declares
        item.assigns = parseResult.assigns
        item.alters = parseResult.alters
        item.uses = parseResult.uses

        if (parseResult.errors.length > 0) {
          if (item.errors === undefined) {
            item.errors = []
          }
          item.errors = item.errors.concat(parseResult.errors)
        }
        code.push({
          codeChunk: item,
          parseResult: parseResult
        })
      } else code.push(item)
    }

    Object.entries(item).forEach(([, i]) => {
      parseItem(i, parameters, code, functionDepth)
    })

    if (isA('Function', item)) {
      --functionDepth
    }
  } else if (Array.isArray(item)) {
    item.forEach(i => parseItem(i, parameters, code, functionDepth))
  }
}

class CodeChunkParseResult {
  public chunkAst: Program | null

  public imports: string[] = []

  public declares: (Variable | Function)[] = []

  public assigns: string[] = []

  public alters: string[] = []

  public uses: string[] = []

  public errors: CodeError[] = []

  private seenIdentifiers: string[] = []

  private possibleVariables: string[] = []

  public constructor(chunkAst: Program | null) {
    this.chunkAst = chunkAst
  }

  private isNameSet(name: string): boolean {
    if (!this.seenIdentifiers.includes(name)) {
      this.seenIdentifiers.push(name)
      return false
    }
    return true
  }

  public addImports(n: string): void {
    if (!this.imports.includes(n)) this.imports.push(n)
  }

  public addPossibleVariable(n: string): void {
    /* If a declaration has a more complex initializer (like a ternary) then we don't know straight away if it's a
     * variable or a function. So the identifier goes in here and if it wasn't computed to be a function declaration
     * after parsing the rest of the code chunk, then set is as a variable.
     */
    if (!this.possibleVariables.includes(n)) this.possibleVariables.push(n)
  }

  public addDeclares(d: Variable | Function): void {
    if (!this.isNameSet(d.name)) this.declares.push(d)
  }

  public addAssigns(n: string): void {
    if (!this.isNameSet(n)) this.assigns.push(n)
  }

  public addAlters(n: string): void {
    if (!this.isNameSet(n)) this.alters.push(n)
  }

  public addUses(n: string): void {
    if (!this.isNameSet(n)) this.uses.push(n)
  }

  public finalize(): void {
    this.possibleVariables.forEach(n => {
      if (!this.seenIdentifiers.includes(n)) this.declares.push(variable(n))
    })
  }
}

function recurseMemberExpression(expr: MemberExpression): string | null {
  if (expr.object.type === 'Identifier') return expr.object.name
  if (expr.object.type === 'MemberExpression')
    return recurseMemberExpression(expr.object)

  return null
}

function parseFunctionDeclaration(
  result: CodeChunkParseResult,
  fn: FunctionDeclaration | FunctionExpression,
  name?: string | null
): void {
  if (name === undefined) {
    name = fn.id !== null ? fn.id.name : null
  }

  if (name === null) return

  const parameters: Parameter[] = []

  if (fn.params !== undefined) {
    fn.params.forEach(p => {
      if (p.type === 'Identifier') parameters.push(parameter(p.name))
      else if (p.type === 'RestElement' && p.argument.type === 'Identifier')
        parameters.push(parameter(p.argument.name, { repeats: true }))
    })
  }

  result.addDeclares(function_(name, { parameters }))
}

function parseVariableDeclaration(
  result: CodeChunkParseResult,
  statement: VariableDeclaration
): void {
  statement.declarations.forEach(declarator => {
    if (declarator.id.type === 'Identifier') {
      if (declarator.init === null || declarator.init.type === 'Literal')
        result.addDeclares(variable(declarator.id.name))
      else {
        result.addPossibleVariable(declarator.id.name)
        parseStatement(result, declarator.init, declarator.id.name)
      }
    }
  })
}

function parseAssignmentExpression(
  result: CodeChunkParseResult,
  statement: AssignmentExpression
): void {
  let assignmentName: string | undefined

  if (statement.left.type === 'Identifier') {
    assignmentName = statement.left.name
    result.addAssigns(statement.left.name)
  } else if (statement.left.type === 'MemberExpression') {
    const name = recurseMemberExpression(statement.left)
    if (name !== null) result.addAlters(name)
  }

  /* an offshoot of only setting assignmentName for Identifiers only (not MemberExpressions) is that functions that
  are declared and assigned to a property of an object won't be parsed, since their name will be like `a.b` so won't be
  a valid function identifier
  */
  parseStatement(result, statement.right, assignmentName)
}

function parseBinaryExpression(
  result: CodeChunkParseResult,
  statement: BinaryExpression
): void {
  if (statement.left.type === 'Identifier') result.addUses(statement.left.name)
  else parseStatement(result, statement.left)

  if (statement.right.type === 'Identifier')
    result.addUses(statement.right.name)
  else parseStatement(result, statement.right)
}

function parseCallExpression(
  result: CodeChunkParseResult,
  statement: CallExpression
): void {
  statement.arguments.forEach(arg => {
    if (arg.type === 'Identifier') result.addUses(arg.name)
    else parseStatement(result, arg)
  })
}

function parseUpdateExpression(
  result: CodeChunkParseResult,
  statement: UpdateExpression
): void {
  if (statement.argument.type === 'Identifier')
    result.addAlters(statement.argument.name)
  else parseStatement(result, statement.argument)
}

function parseImportExpression(
  result: CodeChunkParseResult,
  statement: ImportDeclaration
): void {
  if (
    statement.source.type === 'Literal' &&
    typeof statement.source.value === 'string'
  ) {
    result.addImports(statement.source.value)
  }
}

function parseWhileStatement(
  result: CodeChunkParseResult,
  statement: WhileStatement | DoWhileStatement
): void {
  parseStatement(result, statement.test)

  parseStatement(result, statement.body)
}

function parseIfStatement(
  result: CodeChunkParseResult,
  statement: IfStatement
): void {
  parseStatement(result, statement.test)
  parseStatement(result, statement.consequent)
  if (statement.alternate !== null) {
    parseStatement(result, statement.alternate)
  }
}

function parseTryStatement(
  result: CodeChunkParseResult,
  statement: TryStatement
): void {
  statement.block.body.forEach(subStatement => {
    parseStatement(result, subStatement)
  })

  if (statement.handler !== null) {
    statement.handler.body.body.forEach(subStatement => {
      parseStatement(result, subStatement)
    })
  }

  if (statement.finalizer !== null) {
    statement.finalizer.body.forEach(subStatement => {
      parseStatement(result, subStatement)
    })
  }
}

function parseExpression(
  result: CodeChunkParseResult,
  statement: ExpressionStatement
): void {
  parseStatement(result, statement.expression)
}

function parseStatement(
  result: CodeChunkParseResult,
  statement: EtreeNode,
  lastParsedVarName?: string
): void {
  switch (statement.type) {
    case 'VariableDeclaration':
      parseVariableDeclaration(result, statement as VariableDeclaration)
      break
    case 'ExpressionStatement':
      parseExpression(result, statement as ExpressionStatement)
      break
    case 'AssignmentExpression':
      parseAssignmentExpression(result, statement as AssignmentExpression)
      break
    case 'BinaryExpression':
      parseBinaryExpression(result, statement as BinaryExpression)
      break
    case 'FunctionDeclaration':
      parseFunctionDeclaration(result, statement as FunctionDeclaration)
      break
    case 'FunctionExpression':
      parseFunctionDeclaration(
        result,
        statement as FunctionExpression,
        lastParsedVarName
      )
      break
    case 'CallExpression':
      parseCallExpression(result, statement as CallExpression)
      break
    case 'UpdateExpression':
      parseUpdateExpression(result, statement as UpdateExpression)
      break
    case 'ImportDeclaration':
      parseImportExpression(result, statement as ImportDeclaration)
      break
    case 'WhileStatement':
    case 'DoWhileStatement':
      // both actually have the same interface so it is OK to cast just to WhileStatement
      parseWhileStatement(result, statement as WhileStatement)
      break
    case 'IfStatement':
    case 'ConditionalExpression':
      // both actually have the same interface so it is OK to cast just to IfStatement
      parseIfStatement(result, statement as IfStatement)
      break
    case 'BlockStatement':
      statement.body.forEach(subStatement => {
        parseStatement(result, subStatement)
      })
      break
    case 'TryStatement':
      parseTryStatement(result, statement)
      break
    case 'EmptyStatement':
    case 'Identifier':
    case 'UnaryExpression':
    case 'Literal':
    case 'ThrowStatement':
      break
    default:
      console.log(statement)
      throw new Error(`Unhandled statement ${statement.type}`)
  }
}

function exceptionToCodeError(error: Error | string): CodeError {
  if (typeof error === 'string') {
    return codeError('Exception', { message: error })
  } else {
    return codeError(error.name, { message: error.message, trace: error.stack })
  }
}

function setCodeError(
  code: CodeChunk | CodeExpression,
  error: Error | string
): void {
  if (code.errors === undefined) {
    code.errors = []
  }
  code.errors.push(exceptionToCodeError(error))
}

function parseCodeChunk(codeChunk: CodeChunk): CodeChunkParseResult {
  let chunkAst: Program | null = null

  try {
    chunkAst = parse(codeChunk.text, { module: true })
  } catch (e) {
    const badParseResult: CodeChunkParseResult = new CodeChunkParseResult(null)
    badParseResult.errors.push(exceptionToCodeError(e))
    return badParseResult
  }

  const parseResult: CodeChunkParseResult = new CodeChunkParseResult(chunkAst)

  chunkAst.body.forEach(statement => {
    parseStatement(parseResult, statement)
  })

  return parseResult
}

/**
 * Execute a `CodeExpression` (which should be a single expression) and set the result onto its `output` property.
 *
 * Uses `eval`, so should only be called with trusted code.
 */
function executeCodeExpression(code: CodeExpression): void {
  try {
    // eslint-disable-next-line no-eval
    code.output = eval(code.text)
  } catch (e) {
    setCodeError(code, e)
  }
}

/**
 * Execute a `CodeChunk` (which might be multiple lines) and set any return values or console output as items in an
 * array on it `outputs` property.
 *
 * Uses `eval`, so should only be called with trusted code.
 */
function executeCodeChunk(code: CodeChunkExecution): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const outputs: any[] = []

  const ast = code.parseResult.chunkAst

  if (ast === null) return

  const chunk = code.codeChunk

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

    let res

    try {
      // @ts-ignore
      // eslint-disable-next-line no-eval
      res = (1, eval)(generatedCode)
    } catch (e) {
      setCodeError(chunk, e)
    }

    console.log = oldCl

    outputs.push(res)

    if (loggedData.length > 0) {
      outputs.push(loggedData)
    }
  })

  chunk.outputs = outputs.filter(o => o !== undefined)
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
      if (es.values !== undefined && !es.values.includes(value))
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
): StringDict {
  const decodedValues: StringDict = {}

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
  const j = JSON.stringify(output, null, 2)

  if (path === '-') {
    console.log(j)
  } else {
    fs.writeFileSync(path, j)
  }
}

/**
 * Execute a document based on arguments from the command line.
 */
async function main(): Promise<void> {
  const cliArgs = getCliArgs()
  const article = JSON.parse(await readInput(cliArgs.inputFile))

  if (!isA('Article', article)) throw TypeError('Not an Article')

  const parameters: Parameter[] = []
  const code: ExecutableCode[] = []

  parseItem(article, parameters, code)

  execute(code, decodeParameters(parameters, cliArgs.parameterValues))

  outputArticle(cliArgs.outputFile, article)
}
