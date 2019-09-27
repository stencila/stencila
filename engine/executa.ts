import * as childProcess from 'child_process'
import { getLogger } from '@stencila/logga'
const lps = require('length-prefixed-stream')

const { spawn } = require('child_process')

const log = getLogger('engine:serve')

interface ExecutorBackend {
  setup(): void

  execute(o: any): Promise<any>
}

class StencilaPythonBackend implements ExecutorBackend {
  private process?: childProcess.ChildProcess

  private stdin: any

  private executionRequests: { [key: number]: any } = {}

  private executionRequestCount: number = 0

  setup(): void {
    this.process = spawn('python3', ['-m', 'stencila.schema', 'listen'])
    if (!this.process) throw new Error('Spawning python3 failed')
    if (
      this.process.stdout === null ||
      this.process.stdin === null ||
      this.process.stderr === null
    )
      throw new Error('STDIO is not set up right')

    this.process.stderr.on('data', (data: Buffer) => {
      log.error(data.toString())
      process.exit(1)
    })

    const decoder = lps.decode()
    this.process.stdout.pipe(decoder)
    decoder.on('data', (response: Buffer) => this.receive(response))

    this.stdin = lps.encode()
    this.stdin.pipe(this.process.stdin)
  }

  receive(json: Buffer, raw: boolean = false) {
    const response = JSON.parse(json.toString())
    const resolve = this.executionRequests[response.id]
    resolve(response.body)
    delete this.executionRequests[response.id]
  }

  async execute(o: any): Promise<any> {
    if (!this.process) {
      throw new Error('Can not execute before setup')
    }

    const id = ++this.executionRequestCount

    const req = {
      id,
      body: o
    }

    const promise = new Promise<any>((resolve, reject) => {
      this.executionRequests[id] = (response: any) => {
        resolve(response)
      }
    })

    this.stdin.write(JSON.stringify(req))

    return promise
  }
}

export default class Executa {
  backends: { [key: string]: ExecutorBackend } = {}

  getBackend(name: string): ExecutorBackend {
    if (this.backends[name] === undefined) {
      switch (name) {
        case 'python':
          this.backends[name] = new StencilaPythonBackend()
          break
        default:
          throw new Error(`Unknown backend '${name}'`)
      }
      this.backends[name].setup()
    }

    return this.backends[name]
  }

  async execute(code: any): Promise<any> {
    const backend = this.getBackend(code.programmingLanguage)

    return await backend.execute(code)
  }
}
