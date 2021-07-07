import { CHANNEL } from '../preload/channels'

import { Document, Error } from 'stencila'

/**
 * The result of calling a function
 *
 * This is similar to a Rust `Error` except that it
 * has potentially more than one error and each is displayed to
 * to the user with varying severity and suggestions for remedial
 * action.
 */
export class Result<T> {
  value?: T
  errors?: Error[]

  constructor(value?: T, errors?: Error[]) {
    ;(this.value = value), (this.errors = errors)
  }

  // Displays errors, returns the value if is defined, throws the
  // last error, the one that the function failed on
  unwrap(): T {
    if (this.errors?.length ?? 0 > 0) console.error(this.errors)
    if (this.value !== undefined) return this.value
    else
      throw Error(this.errors?.[this.errors?.length]?.type ?? 'Unknown error')
  }
}

/**
 * Abstract base class for clients
 * 
 * Alternative clients need to implement the `call` method which
 * send the RPC request to the "server"
 */
abstract class Client {
  abstract call<T>(method: string, ...params: unknown[]): Promise<Result<T>>

  documentsOpen = (path: string, format?: string) =>
    this.call<Document>('documentsOpen', path, format)
}

/**
 * A client that uses the Electron IPC mechanism
 */
export class ElectronClient extends Client {
  async call<T>(method: string, ...params: unknown[]): Promise<Result<T>> {
    let { value, errors } = (await window.api.invoke(CHANNEL.RPC_CALL, {
      method,
      params,
    })) as Result<T>
    return new Result<T>(value, errors)
  }
}

export const client = new ElectronClient()
