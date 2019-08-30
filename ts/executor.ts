import getStdin from 'get-stdin'
import minimist from 'minimist'
import { parse } from 'meriyah'
import { generate } from 'astring'
import { performance } from 'perf_hooks'

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
  TryStatement,
  ForStatement,
  ForInStatement,
  ForOfStatement,
  ConditionalExpression,
  ArrayExpression,
  ObjectExpression,
  LogicalExpression,
  SwitchStatement
} from 'meriyah/dist/estree'

// eslint-disable-next-line @typescript-eslint/no-floating-promises
if (process.env.JEST_WORKER_ID === undefined) main()

interface StringDict {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any
}

interface CliArgs {
  inputFile: string

  outputFile: string

  parameterValues: StringDict
}

export interface CodeChunkExecution {
  codeChunk: CodeChunk
  parseResult: CodeChunkParseResult
}

export type ExecutableCode = CodeChunkExecution | CodeExpression

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
export function execute(
  code: ExecutableCode[],
  parameterValues: StringDict
): void {
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
export function parseItem(
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
        item.reads = parseResult.reads

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

  public reads: string[] = []

  public errors: CodeError[] = []

  public functionDeclarationDepth = 0

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
    if (this.functionDeclarationDepth > 0) return

    if (!this.imports.includes(n)) this.imports.push(n)
  }

  public addPossibleVariable(n: string): void {
    /* If a declaration has a more complex initializer (like a ternary) then we don't know straight away if it's a
     * variable or a function. So the identifier goes in here and if it wasn't computed to be a function declaration
     * after parsing the rest of the code chunk, then set is as a variable.
     */
    if (this.functionDeclarationDepth > 0) return

    if (!this.possibleVariables.includes(n)) this.possibleVariables.push(n)
  }

  public addDeclares(d: Variable | Function): void {
    if (this.functionDeclarationDepth > 0) return

    if (!this.isNameSet(d.name)) this.declares.push(d)
  }

  public addAssigns(n: string): void {
    if (this.functionDeclarationDepth > 0) return

    const useIndex = this.uses.indexOf(n)

    if (useIndex !== -1) {
      this.uses.splice(useIndex, 1)
      this.assigns.push(n)
      return // it will already exist in the isNameSet array
    }

    if (!this.isNameSet(n)) this.assigns.push(n)
  }

  public addAlters(n: string): void {
    if (this.functionDeclarationDepth > 0) return

    if (!this.isNameSet(n)) this.alters.push(n)
  }

  public addUses(n: string): void {
    if (this.functionDeclarationDepth > 0) return

    if (!this.isNameSet(n)) this.uses.push(n)
  }

  public addReads(f: string): void {
    if (!this.reads.includes(f)) this.reads.push(f)
  }

  public finalize(): void {
    this.possibleVariables.forEach(n => {
      if (!this.seenIdentifiers.includes(n)) this.declares.push(variable(n))
    })
  }
}

function recurseMemberExpression(
  expr: MemberExpression,
  result?: CodeChunkParseResult
): string | null {
  // pass result to store Identifiers found while recursing
  if (result !== undefined) {
    if (expr.property.type === 'MemberExpression') {
      recurseMemberExpression(expr.property, result)
    } else if (expr.property.type === 'Identifier') {
      result.addUses(expr.property.name)
    }
  }

  if (expr.object.type === 'Identifier') {
    if (result !== undefined) result.addUses(expr.object.name)
    return expr.object.name
  }
  if (expr.object.type === 'MemberExpression') {
    return recurseMemberExpression(expr.object)
  }

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
      if (p.type === 'Identifier')
        parameters.push(
          parameter(p.name, { required: true, repeats: false, extends: false })
        )
      else if (p.type === 'RestElement' && p.argument.type === 'Identifier')
        parameters.push(
          parameter(p.argument.name, {
            required: false,
            repeats: true,
            extends: false
          })
        )
    })
  }

  result.addDeclares(function_(name, { parameters }))

  if (fn.body !== undefined && fn.body !== null) {
    ++result.functionDeclarationDepth
    parseStatement(result, fn.body)
    --result.functionDeclarationDepth
  }
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

  if (statement.right.type === 'Identifier') {
    result.addUses(statement.right.name)
  } else {
    /* an offshoot of only setting assignmentName for Identifiers only (not MemberExpressions) is that functions that
    are declared and assigned to a property of an object won't be parsed, since their name will be like `a.b` so won't be
    a valid function identifier
    */
    parseStatement(result, statement.right, assignmentName)
  }
}

function parseBinaryOrLogicalExpression(
  result: CodeChunkParseResult,
  statement: BinaryExpression | LogicalExpression
): void {
  if (statement.left.type === 'Identifier') result.addUses(statement.left.name)
  else parseStatement(result, statement.left)

  if (statement.right.type === 'Identifier')
    result.addUses(statement.right.name)
  else parseStatement(result, statement.right)
}

function isFileRead(statement: CallExpression): [boolean, string | null] {
  let calleeName: string
  if (statement.callee.name !== undefined) {
    calleeName = statement.callee.name
  } else if (
    statement.callee.type === 'MemberExpression' &&
    statement.callee.property.type === 'Identifier'
  ) {
    calleeName = statement.callee.property.name
  } else return [false, null]
  return [
    calleeName === 'readFileSync' ||
      calleeName === 'readFile' ||
      calleeName === 'open',
    calleeName
  ]
}

function parseFileReadExpression(
  result: CodeChunkParseResult,
  statement: CallExpression,
  calleeName: string
): void {
  if (statement.arguments.length === 0) return

  const arg = statement.arguments[0]

  if (arg.type !== 'Literal' || typeof arg.value !== 'string') {
    return
  }
  // file name is a string so we can determine the path without executing

  if (calleeName === 'open' && statement.arguments.length >= 2) {
    // if for some reason open() only has one arg assume it is a read. this is invalid code though.
    const modeObj = statement.arguments[1]
    if (modeObj.type !== 'Literal' || typeof modeObj.value !== 'string') return

    const mode = modeObj.value

    if (mode.indexOf('r') === -1 && mode.indexOf('+') === -1) return
  }

  result.addReads(arg.value)
}

function parseCallExpression(
  result: CodeChunkParseResult,
  statement: CallExpression
): void {
  const [isRead, calleeName] = isFileRead(statement)

  if (isRead) {
    parseFileReadExpression(result, statement, calleeName as string) // typecasting as calleeName is always non-null if isRead is true
  }

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

function parseConditionalStatement(
  result: CodeChunkParseResult,
  statement: IfStatement | ConditionalExpression
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

function parseForStatement(
  result: CodeChunkParseResult,
  statement: ForStatement | ForInStatement | ForOfStatement
): void {
  if (
    statement.type === 'ForInStatement' ||
    statement.type === 'ForOfStatement'
  ) {
    if (statement.right.type === 'Identifier')
      result.addUses(statement.right.name)
  }

  // ignore statement.init, statement.test and statement.update as variables set here probably aren't intended for use
  // throughout the code. This stance can be revised if it turns out users are setting vars in for() to use later
  parseStatement(result, statement.body)
}

function parseArrayExpression(
  result: CodeChunkParseResult,
  statement: ArrayExpression
): void {
  statement.elements.forEach(el => {
    if (el.type === 'Identifier') result.addUses(el.name)
    else parseStatement(result, el)
  })
}

function parseObjectExpression(
  result: CodeChunkParseResult,
  statement: ObjectExpression
): void {
  statement.properties.forEach(p => {
    if (p.type === 'Property') {
      if (p.key.type === 'Identifier') result.addUses(p.key.name)
      else if (p.key.type === 'MemberExpression') {
        recurseMemberExpression(p.key, result)
      }

      if (p.value.type === 'Identifier') result.addUses(p.value.name)
      else if (p.value.type === 'MemberExpression') {
        recurseMemberExpression(p.value, result)
      }
    }
  })
}

function parseMemberExpression(
  result: CodeChunkParseResult,
  statement: MemberExpression
): void {
  const memberName = recurseMemberExpression(statement)
  if (memberName !== null) result.addUses(memberName)
}

function parseSwitchStatement(
  result: CodeChunkParseResult,
  statement: SwitchStatement
): void {
  parseStatement(result, statement.discriminant)
  statement.cases.forEach(c => {
    if (c.test !== null) {
      if (c.test.type === 'Identifier') {
        result.addUses(c.test.name)
      } else parseStatement(result, c.test)
    }
    c.consequent.forEach(subStatement => parseStatement(result, subStatement))
  })
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
      parseVariableDeclaration(result, statement)
      break
    case 'ExpressionStatement':
      parseExpression(result, statement)
      break
    case 'AssignmentExpression':
      parseAssignmentExpression(result, statement)
      break
    case 'BinaryExpression':
    case 'LogicalExpression':
      parseBinaryOrLogicalExpression(result, statement)
      break
    case 'FunctionDeclaration':
      parseFunctionDeclaration(result, statement)
      break
    case 'FunctionExpression':
      parseFunctionDeclaration(
        result,
        statement as FunctionExpression,
        lastParsedVarName
      )
      break
    case 'CallExpression':
      parseCallExpression(result, statement)
      break
    case 'UpdateExpression':
      parseUpdateExpression(result, statement)
      break
    case 'ImportDeclaration':
      parseImportExpression(result, statement)
      break
    case 'WhileStatement':
    case 'DoWhileStatement':
      parseWhileStatement(result, statement)
      break
    case 'IfStatement':
    case 'ConditionalExpression':
      parseConditionalStatement(result, statement)
      break
    case 'BlockStatement':
      statement.body.forEach(subStatement => {
        parseStatement(result, subStatement)
      })
      break
    case 'TryStatement':
      parseTryStatement(result, statement)
      break
    case 'ForStatement':
    case 'ForInStatement':
    case 'ForOfStatement':
      parseForStatement(result, statement)
      break
    case 'ArrayExpression':
      parseArrayExpression(result, statement)
      break
    case 'ObjectExpression':
      parseObjectExpression(result, statement)
      break
    case 'MemberExpression':
      parseMemberExpression(result, statement)
      break
    case 'SwitchStatement':
      parseSwitchStatement(result, statement)
      break
    case 'EmptyStatement':
    case 'Identifier':
    case 'UnaryExpression':
    case 'Literal':
    case 'ThrowStatement':
    case 'ReturnStatement':
    case 'BreakStatement':
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

export function parseCodeChunk(codeChunk: CodeChunk): CodeChunkParseResult {
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

  let duration = 0

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

    const execStart = performance.now()

    try {
      // @ts-ignore
      // eslint-disable-next-line no-eval
      res = (1, eval)(generatedCode)
    } catch (e) {
      setCodeError(chunk, e)
    }

    duration += performance.now() - execStart

    console.log = oldCl

    outputs.push(res)

    if (loggedData.length > 0) {
      outputs.push(loggedData)
    }
  })

  chunk.duration = duration
  chunk.outputs = outputs.filter(o => o !== undefined)
}

/**
 * Decode a parameter (e.g. read from the command line, a string) into a value based on the `schema` of the `parameter`.
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
function decodeParameter(parameter: Parameter, value?: string): any {
  if (isA('ConstantSchema', parameter.schema)) return parameter.schema.value

  if (value === undefined) {
    if (parameter.required === true)
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
export function decodeParameters(
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
