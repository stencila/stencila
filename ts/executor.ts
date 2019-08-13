import getStdin from 'get-stdin'
import minimist from 'minimist'
import fs from 'fs'
import {
  Article,
  CodeChunk,
  CodeExpression,
  ConstantSchema,
  EnumSchema,
  Parameter
} from './types'
import { isA } from './util'

const { _, ...readParams } = minimist(process.argv.slice(2), {})

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

main()

class Executor {
  public parameters: Parameter[] = []

  public code: (CodeChunk | CodeExpression)[] = []

  public parse(source: Article): void {
    this.handleItem(source)
  }

  public execute(parameterValues: { [key: string]: any }): void {
    Object.entries(parameterValues).forEach(([key, value]) => {
      // @ts-ignore
      global[key] = value
    })

    this.code.forEach(c => {
      if (isA('CodeChunk', c)) {
        this.executeCodeChunk(c)
      } else if (isA('CodeExpression', c)) {
        this.executeCodeExpression(c)
      }
    })
  }

  private handleItem(item: any) {
    if (isA('Entity', item) || item instanceof Object) {
      if (isA('Parameter', item)) {
        this.parameters.push(item)
      } else if (
        (isA('CodeChunk', item) || isA('CodeExpression', item)) &&
        item.language === 'javascript'
      ) {
        this.code.push(item)
      }

      Object.entries(item).forEach(([, value]) => this.handleItem(value))
    } else if (Array.isArray(item)) {
      item.forEach(this.handleItem)
    }
  }

  private executeCodeChunk(code: CodeChunk): void {
    const outputs: any[] = []

    code.text.split('\n').map(statement => {
      const oldCl = console.log

      let loggedData = ''

      console.log = (s: string) => {
        loggedData += s
      }

      // eslint-disable-next-line no-eval
      outputs.push(eval(statement))

      console.log = oldCl

      if (loggedData.length > 0) {
        outputs.push(loggedData)
      }
    })
    code.outputs = outputs.filter(o => o !== undefined)
  }

  private executeCodeExpression(code: CodeExpression): void {
    // eslint-disable-next-line no-eval
    code.output = eval(code.text)
  }
}

function decodeParameters(
  parameters: Parameter[],
  values: { [key: string]: string }
): { [key: string]: any } {
  const decodedValues: { [key: string]: any } = {}

  parameters.forEach(p => {
    const raw = values[p.name]
    let value

    if (raw === undefined && !isA('ConstantSchema', p.schema)) {
      if (p.default === undefined)
        throw new Error(`No value or default found for parameter ${p.name}`)
      value = p.default
    } else if (p.schema !== undefined) {
      switch (p.schema.type) {
        case 'ConstantSchema':
          value = (p.schema as ConstantSchema).value
          break
        case 'ArraySchema':
        case 'TupleSchema':
          value = JSON.parse(raw)
          break
        case 'BooleanSchema':
          const rawLower = raw.toLowerCase()
          value =
            rawLower === 'true' ||
            rawLower === 't' ||
            rawLower === 'yes' ||
            rawLower === '1'
          break
        case 'EnumSchema':
          const es = p.schema as EnumSchema
          if (es.values !== undefined && es.values.indexOf(raw) === -1)
            throw new Error(`${raw} not found in enum values for ${p.name}`)
          value = raw
          break
        case 'IntegerSchema':
          value = parseInt(raw)
          if (isNaN(value)) throw new Error(`$[raw} is not a valid integer`)
          break
        case 'NumberSchema':
          value = parseFloat(raw)
          if (isNaN(value)) throw new Error(`$[raw} is not a valid float`)
          break
        default:
          value = raw
      }
    }
    decodedValues[p.name] = value === undefined ? raw : value
  })

  return decodedValues
}

async function readInput(path: string): Promise<string> {
  if (path === '-') {
    return getStdin()
  }

  return fs.readFileSync(path, 'utf8')
}

async function main() {
  const article = JSON.parse(await readInput(inFile))

  if (!isA('Article', article)) throw TypeError('Not an Article')

  const e = new Executor()
  e.parse(article)
  e.execute(decodeParameters(e.parameters, readParams))
  console.log(article)
}
