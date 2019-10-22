import getStdin from 'get-stdin'
import minimist from 'minimist'
import { Parser } from 'acorn'

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
} from 'estree'
import * as os from 'os'
import * as path from 'path'
import log from './log'

// eslint-disable-next-line @typescript-eslint/no-var-requires
const lps = require('length-prefixed-stream')

const EXECUTORS_DIR_NAME = 'executors'
const MANIFEST_FILE_NAME = 'javascript.json'

if (process.env.JEST_WORKER_ID === undefined)
  main()
    .then()
    .catch(log.error)

interface StringDict {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  [key: string]: any
}

interface CliArgs {
  command: string

  inputFile: string

  outputFile: string

  parameterValues: StringDict
}

/**
 * To prevent having to build an AST from the `CodeChunk` twice (once when it is parsed and again when executed) store
 * the `CodeChunk` and `CodeChunkParseResult` (which contains the AST) together for execution.
 */
export interface CodeChunkExecution {
  codeChunk: CodeChunk
  parseResult: CodeChunkParseResult
}

export type ExecutableCode = CodeChunkExecution | CodeExpression

/**
 * Process argv to get the source and destination for the document being executed
 */
function getCliArgs(): CliArgs {
  const { _, ...parameterValues } = minimist(process.argv.slice(2), {})

  if (_.length === 0) throw new Error('No command supplied')

  const command = _[0]

  switch (command) {
    case 'execute':
    case 'compile':
    case 'listen':
    case 'register':
    case 'deregister':
      break
    default:
      throw new Error(`Unknown command ${command}`)
  }

  let inputFile = '-'
  let outputFile = '-'

  if (_.length === 1) {
    // read from stdin and output to stdout
  }

  if (_.length >= 2) {
    // read from provided file arg (which might be -), then output to stdout
    inputFile = _[1]

    if (_.length >= 3) {
      // read from file, output to a file. but the args might both be -
      outputFile = _[2]
    }
  }

  return {
    command,
    inputFile,
    outputFile,
    parameterValues
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
 * Add `imports` to the `CodeChunk.imports` array if they aren't already there. If the existing imports array contains
 * the empty string semaphore then no more imports should be added.
 */
function setCodeChunkImports(code: CodeChunk, imports: string[]): void {
  if (code.imports === undefined) {
    code.imports = imports
    return
  }

  // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
  if (code.imports.includes('')) return

  // Can't do any kind of de-dupe magic as code.imports might contain types other than strings.
  imports.forEach(im => {
    // Typescript seems to forget that this has already been checked to not be undefined
    if (code.imports === undefined) return

    // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
    if (!code.imports.includes(im)) code.imports.push(im)
  })
}

/**
 * Set the properties from a `CodeChunkParseResult` onto a `CodeChunk` (hopefully the one from which the parse result
 * was generated).
 */
function setCodeChunkProperties(
  codeChunk: CodeChunk,
  parseResult: CodeChunkParseResult
): void {
  setCodeChunkImports(codeChunk, parseResult.imports)
  codeChunk.declares = parseResult.declares
  codeChunk.assigns = parseResult.assigns
  codeChunk.alters = parseResult.alters
  codeChunk.uses = parseResult.uses
  codeChunk.reads = parseResult.reads

  if (parseResult.errors.length > 0) {
    if (codeChunk.errors === undefined) {
      codeChunk.errors = []
    }
    codeChunk.errors = codeChunk.errors.concat(parseResult.errors)
  }
}

/**
 * Traverse a `Node` hierarchy and push any `Parameter` that is found onto the `parameters` array,
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
      item.programmingLanguage === 'javascript'
    ) {
      if (isA('CodeChunk', item)) {
        const parseResult = parseCodeChunk(item)
        setCodeChunkProperties(item, parseResult)
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

/**
 * Store the results of parsing a `CodeChunk`. Usually these attributes are then stored on the `CodeChunk` itself.
 */
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

/**
 * Recurse through an expression like a.b.c or a[b][c]. Return the actual variable to which the members belong (in this
 * case, `a`).
 *
 * If the `result` parameter is supplied (usually only in the array access case), then store identifiers found along
 * the way into the `uses` array. In this case, `b` and `c` would be stored.
 */
function recurseMemberExpression(
  expr: MemberExpression,
  result?: CodeChunkParseResult
): string | null {
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

/**
 * Parse a `function functionName() {...}` block (FunctionDeclaration) or
 * `(var | const | let) functionName = function () {...}` block (FunctionExpression). In the latter case the name of
 * the function is not part of the `fn` object (as it is parsed as a VariableDeclaration in a previous call) so it needs
 * to be passed in as the `name` parameter.
 */
function parseFunctionDeclaration(
  result: CodeChunkParseResult,
  fn: FunctionDeclaration | FunctionExpression,
  name?: string | null
): void {
  if (name === undefined) {
    name = fn.id !== undefined && fn.id !== null ? fn.id.name : null
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

/**
 * Parse a VariableDeclaration statement (`const`/`let`/`var`) and add it to the result's `declares` array. In some
 * cases (e.g. function declarations) the declaration is not added immediately, instead it is added to the
 * `possibleVariables` array in the result. If it is a function declaration then the `parseFunctionDeclaration` function
 * will take care of adding it to the `declares` array as a `Function`. If it is some other initializer (like a member
 * expression) then upon calling `finalize()` on the `result` when parsing finishes, the found name with be added to the
 * `declares` array as a `Variable`.
 */
function parseVariableDeclaration(
  result: CodeChunkParseResult,
  statement: VariableDeclaration
): void {
  statement.declarations.forEach(declarator => {
    if (declarator.id.type === 'Identifier') {
      if (
        declarator.init === null ||
        declarator.init === undefined ||
        declarator.init.type === 'Literal'
      )
        result.addDeclares(variable(declarator.id.name))
      else {
        result.addPossibleVariable(declarator.id.name)
        parseStatement(result, declarator.init, declarator.id.name)
      }
    }
  })
}

/**
 * Parse a variable assignment. If the left side is a simple variable (e.g. `a = 4` or `b = c.d`) it will be added to
 * the result's `assigns` array. If it is a member expression (e.g. `a.b = 5` or `c[d][e] = 6`) then the statement will
 * be recursed to find the actual variable being modified (`a` or `c` respectively) and this will be added to the
 * `alters` array. In the latter (array) case then variables used to access elements (`d` and `e`) will be added to the
 * `uses` array.
 */
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
    are declared and assigned to a property of an object won't be parsed, since their name will be like `a.b` so won't
    be a valid function identifier
    */
    parseStatement(result, statement.right, assignmentName)
  }
}

/**
 * Parses a Binary (`a + b`, `d - e`, etc) or logical (`a && b`, `c || d`, etc) adding the variables it uses to the
 * result's `uses` array.
 */
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

/**
 * For a function call, return its name if it is a known (potential) file reading function. Otherwise return null.
 * This takes into account a module name in the call, e.g `fs.readFile` and `readFile` both will return `readFile`.
 *
 * The known function calls for file reading are: `readFile`, `readFileSync` and `open`. Note that other checks are
 * performed later to verify that a call is actually a read, as `open` can be called with modes (e.g. 'w') that are not.
 */
function fileReadFunctionName(statement: CallExpression): string | null {
  let calleeName: string
  if ('name' in statement.callee && statement.callee.name !== undefined) {
    calleeName = statement.callee.name
  } else if (
    statement.callee.type === 'MemberExpression' &&
    statement.callee.property.type === 'Identifier'
  ) {
    calleeName = statement.callee.property.name
  } else return null

  switch (calleeName) {
    case 'readFileSync':
    case 'readFile':
    case 'open':
      return calleeName
    default:
      return null
  }
}

/**
 * Parse a function call that should have previously been determined to be a `fileReadFunctionName`. If it is `open`,
 * check for a `mode` that indicates a read and not just a write. Otherwise, assume the first arg to the function is a
 * file name or path and add it to the result's `reads` array.
 */
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

/**
 * Parse a function call and add any identifier parameters it uses to the result's `uses` array.
 *
 * For example: `callFunc(a, 'b', true, c)`, `a`, `c` will be added to `uses`.
 */
function parseCallExpression(
  result: CodeChunkParseResult,
  statement: CallExpression
): void {
  const calleeName = fileReadFunctionName(statement)

  if (calleeName !== null) {
    parseFileReadExpression(result, statement, calleeName)
  }

  statement.arguments.forEach(arg => {
    if (arg.type === 'Identifier') result.addUses(arg.name)
    else parseStatement(result, arg)
  })
}

/**
 * Add a variable that is updated in place (`a++`, `b--`, etc) to the result's `alters` array.
 */
function parseUpdateExpression(
  result: CodeChunkParseResult,
  statement: UpdateExpression
): void {
  if (statement.argument.type === 'Identifier')
    result.addAlters(statement.argument.name)
  else parseStatement(result, statement.argument)
}

/**
 * Parse an `import` statement in a variety of formats and add its module (not the imported name) to the result's
 * `imports` array. This is designed to find the modules the code relies upon rather than the individual functions it
 * imports from them.
 */
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

/**
 * Parse a `while (...) {...}` or `do { ... } while (...)` loop, adding the variables used in the condition to the
 * result's `uses` array.
 */
function parseWhileStatement(
  result: CodeChunkParseResult,
  statement: WhileStatement | DoWhileStatement
): void {
  parseStatement(result, statement.test)

  parseStatement(result, statement.body)
}

/**
 * Parse an `if` statement or ternary expression, adding the variables it uses to the result's `uses` array.
 */
function parseConditionalStatement(
  result: CodeChunkParseResult,
  statement: IfStatement | ConditionalExpression
): void {
  parseStatement(result, statement.test)
  parseStatement(result, statement.consequent)
  if (statement.alternate !== null && statement.alternate !== undefined) {
    parseStatement(result, statement.alternate)
  }
}

/**
 * Parse a `try` statement, (basically just recursing into the body/handler/finalizer blocks).
 */
function parseTryStatement(
  result: CodeChunkParseResult,
  statement: TryStatement
): void {
  statement.block.body.forEach(subStatement => {
    parseStatement(result, subStatement)
  })

  if (statement.handler !== null && statement.handler !== undefined) {
    statement.handler.body.body.forEach(subStatement => {
      parseStatement(result, subStatement)
    })
  }

  if (statement.finalizer !== null && statement.finalizer !== undefined) {
    statement.finalizer.body.forEach(subStatement => {
      parseStatement(result, subStatement)
    })
  }
}

/**
 * Parse a `for(.. .; ...; ...) {...}`, `for(... of ...) {...}` or `for(... in ...) {...}` loop. Adds the variables
 * used in a ForIn or ForOf to a result's `uses` array. A normal `for` loop's init/test/update statements are not parsed
 * because it is assumed that these variables are specific only to the for loop and will be out of scope when it
 * completes.
 */
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

/**
 * Parse an array, added any identifiers found in initialization to the result's `uses` array.
 *
 * e.g `[a, b, 1, 'foo', someFunc(d)]` adds `a`, `b` and `d`.
 */
function parseArrayExpression(
  result: CodeChunkParseResult,
  statement: ArrayExpression
): void {
  statement.elements.forEach(el => {
    if (el.type === 'Identifier') result.addUses(el.name)
    else parseStatement(result, el)
  })
}

/**
 * Parse an object, added any identifiers found in initialization to the result's `uses` array.
 *
 * e.g `{a, b: c, d: 1, 'e': 'foo'}` adds `a`, `b`, `c` and `d`. Note: this might not be strictly correct due to the
 * way object keys could be strings or literals.
 */
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

/**
 * Recurse a MemberExpression (e.g. a.b.c.d) and add the actual variable to the result's `uses` array. In this case,
 * just `a`.
 */
function parseMemberExpression(
  result: CodeChunkParseResult,
  statement: MemberExpression
): void {
  const memberName = recurseMemberExpression(statement)
  if (memberName !== null) result.addUses(memberName)
}

/**
 * Parse a `switch` statement, adding identifiers used in tests to the result's `uses` array.
 *
 * `switch (a) case b: doSomething(); break; default: break;` will add the variables `a` and `b`.
 */
function parseSwitchStatement(
  result: CodeChunkParseResult,
  statement: SwitchStatement
): void {
  parseStatement(result, statement.discriminant)
  statement.cases.forEach(c => {
    if (c.test !== null && c.test !== undefined) {
      if (c.test.type === 'Identifier') {
        result.addUses(c.test.name)
      } else parseStatement(result, c.test)
    }
    c.consequent.forEach(subStatement => parseStatement(result, subStatement))
  })
}

/**
 * An `ExpressionStatement` is a container for something like a `BinaryExpression` or `LogicalExpression`, so pass that
 * contained expression to the main parser.
 */
function parseExpression(
  result: CodeChunkParseResult,
  statement: ExpressionStatement
): void {
  parseStatement(result, statement.expression)
}

/**
 * Tthis is the general parser function which switches on the statement type and delegates to the appropriate parser
 * function. It returns nothing, instead passing around the `result` variable to store the parsed properties.
 */
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

/**
 * Transform an `Error` or `string` (containing an error message` to a new `CodeError` entity.
 */
function exceptionToCodeError(error: Error | string): CodeError {
  if (typeof error === 'string') {
    return codeError('Exception', { message: error })
  } else {
    return codeError(error.name, { message: error.message, trace: error.stack })
  }
}

/**
 * Shortcut function to add a `CodeError` to the code entity's `errors` property. This will create the `errors` array on
 * the code entity if it is `undefined`.
 */
function setCodeError(
  code: CodeChunk | CodeExpression,
  error: Error | string
): void {
  if (code.errors === undefined) {
    code.errors = []
  }
  code.errors.push(exceptionToCodeError(error))
}

/**
 * Parse a `CodeChunk.text` property into an AST then iterate through that to find out things about the code.
 * The `CodeChunkParseResult` is intended as temporary storage and the attributes should be assigned to a `CodeChunk`'s
 * matching properties for output as part of an executed document.
 */
export function parseCodeChunk(codeChunk: CodeChunk): CodeChunkParseResult {
  let chunkAst: Program | null = null

  try {
    chunkAst = (Parser.parse(codeChunk.text, {
      sourceType: 'module'
    }) as unknown) as Program
  } catch (e) {
    const badParseResult: CodeChunkParseResult = new CodeChunkParseResult(null)
    badParseResult.errors.push(exceptionToCodeError(e))
    return badParseResult
  }

  const parseResult: CodeChunkParseResult = new CodeChunkParseResult(
    chunkAst as Program
  )

  chunkAst.body.forEach(statement => {
    parseStatement(parseResult, statement)
  })

  parseResult.finalize()
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
 * Execute a single CodeChunk or CodeExpression with the given parameters
 */
function executeCodeItem<T extends CodeChunk | CodeExpression>(
  code: T,
  parameterValues: StringDict
): T {
  let toExecute: ExecutableCode

  if (isA('CodeChunk', code)) {
    const parseResult = parseCodeChunk(code)
    setCodeChunkProperties(code, parseResult)
    toExecute = { codeChunk: code, parseResult }
  } else {
    toExecute = code as CodeExpression
  }

  execute([toExecute], parameterValues)

  return code
}

/**
 * Generate the path to the Stencila executors directory.
 */
function getExecutorsDir(): string {
  let stencilaHome: string
  switch (os.platform()) {
    case 'darwin':
      stencilaHome = path.join(
        process.env.HOME !== undefined ? process.env.HOME : '',
        'Library',
        'Application Support',
        'Stencila'
      )
      break
    case 'linux':
      stencilaHome = path.join(
        process.env.HOME !== undefined ? process.env.HOME : '',
        '.stencila'
      )
      break
    case 'win32': // is 'win32' even on 64 bit windows systems
      stencilaHome = path.join(
        process.env.APPDATA !== undefined ? process.env.APPDATA : '',
        'Stencila'
      )
      break
    default:
      stencilaHome = path.join(
        process.env.HOME !== undefined ? process.env.HOME : '',
        'stencila'
      )
  }
  return path.join(stencilaHome, EXECUTORS_DIR_NAME)
}

/**
 * Generate the path to the manifest file for this executor.
 */
function getManifestFilePath(): string {
  return path.join(getExecutorsDir(), MANIFEST_FILE_NAME)
}

/**
 * Write out a manifest JSON file for use with Executa automatically detecting execution engines
 */
function register(): void {
  fs.mkdirSync(getExecutorsDir(), { recursive: true })

  const manifest = {
    capabilities: {
      execute: {
        type: 'object',
        required: ['node'],
        properties: {
          node: {
            type: 'object',
            required: ['type', 'programmingLanguage'],
            properties: {
              type: {
                enum: ['CodeChunk', 'CodeExpression']
              },
              programmingLanguage: {
                enum: ['javascript']
              }
            }
          }
        }
      }
    },
    addresses: {
      stdio: {
        type: 'stdio',
        command: 'node',
        args: [__filename, 'listen']
      }
    }
  }
  const manifestPath = getManifestFilePath()
  fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2))
  log.info(`Manifest saved to '${manifestPath}'`)
}

/**
 * Remove the manifest JSON file, if it exists.
 */
function deregister(): void {
  const manifestPath = getManifestFilePath()
  if (fs.existsSync(manifestPath)) {
    fs.unlinkSync(manifestPath)
    log.info(`Deleted manifest at path '${manifestPath}'`)
  } else log.warn(`Not deregistering as file '${manifestPath}' does not exist`)
}

/**
 * Starts a looping listener that reads from STDIN and writes to STDOUT and executes code it receives. Communication is
 * done with LPS protocol.
 */
function listen(): void {
  const decode = lps.decode()
  process.stdin.pipe(decode)

  const encode = lps.encode()
  encode.pipe(process.stdout)

  decode.on('data', async (json: Buffer) => {
    const request = JSON.parse(json.toString())
    const response: { [key: string]: string | string[] | null } = {
      jsonrpc: '2.0',
      id: request.id
    }

    if (request.method !== 'execute') {
      response.error = `Invalid method "${request.method}"`
      response.result = null
    } else {
      const codeNode = request.params.node
      executeCodeItem(codeNode, {})
      response.result = [codeNode]
    }

    encode.write(JSON.stringify(response))
  })
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
      // eslint-disable-next-line @typescript-eslint/strict-boolean-expressions
      if (es.values !== undefined && !es.values.includes(value))
        throw new Error(
          `${value} not found in enum values for ${parameter.name}`
        )
      return value
    case 'IntegerSchema':
      const intValue = parseInt(value)
      if (isNaN(intValue)) throw new Error(`${value} is not a valid integer`)
      return intValue
    case 'NumberSchema':
      const floatValue = parseFloat(value)
      if (isNaN(floatValue)) throw new Error(`${value} is not a valid float`)
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

  if (cliArgs.command === 'compile' || cliArgs.command === 'execute') {
    const article = JSON.parse(await readInput(cliArgs.inputFile))

    if (!isA('Article', article)) throw TypeError('Not an Article')

    const parameters: Parameter[] = []
    const code: ExecutableCode[] = []
    parseItem(article, parameters, code)

    if (cliArgs.command === 'execute')
      execute(code, decodeParameters(parameters, cliArgs.parameterValues))

    outputArticle(cliArgs.outputFile, article)
  } else if (cliArgs.command === 'listen') {
    listen()
  } else if (cliArgs.command === 'register') {
    register()
  } else if (cliArgs.command === 'deregister') {
    deregister()
  }
}
