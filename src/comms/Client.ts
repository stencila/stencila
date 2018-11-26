import JsonRpcError from './JsonRpcError'
import JsonRpcRequest from './JsonRpcRequest'
import JsonRpcResponse from './JsonRpcResponse'
import Processor, { ProcessorManifest } from '../Processor'
import Thing from '../types/Thing'

/**
 * An instance of the base `Processor` class used to import
 * the result of remove calls as a `Thing`.
 */
const processor = new Processor()

/**
 * A base client class which acts as a proxy to a remote `Processor`.
 * Implements aynchronous, proxy methods for `Processor` methods `compile`, `execute`, etc.
 * Those methods send JSON-RPC requests to a `Server` that is serving the remote processor.
 */
export default abstract class Client {

  /**
   * A map of requests to which responses can be paired against
   */
  private requests: {[key: number]: (response: JsonRpcRequest) => void } = {}

  /**
   * Proxy to the remote `Processor`'s `manifest` method
   */
  async manifest (): Promise<ProcessorManifest> {
    return this.call<ProcessorManifest>('manifest')
  }

  /**
   * Proxy to the remote `Processor`'s `import` method
   */
  async import (thing: string | object | Thing, format: string= 'application/ld+json'): Promise<Thing> {
    return this.call<Thing>('import', thing, format)
  }

  /**
   * Proxy to the remote `Processor`'s `export` method
   */
  async export (thing: string | object | Thing, format: string= 'application/ld+json'): Promise<string> {
    return this.call<string>('export', thing, format)
  }

  /**
   * Proxy to the remote `Processor`'s `convert` method
   */
  async convert (thing: string, from: string= 'application/ld+json', to: string= 'application/ld+json'): Promise<string> {
    return this.call<string>('convert', thing, from, to)
  }

  /**
   * Proxy to the remote `Processor`'s `compile` method
   */
  async compile (thing: string | object | Thing, format: string= 'application/ld+json'): Promise<Thing> {
    return this.call<Thing>('compile', thing, format)
  }

  /**
   * Proxy to the remote `Processor`'s `build` method
   */
  async build (thing: string | object | Thing, format: string= 'application/ld+json'): Promise<Thing> {
    return this.call<Thing>('build', thing, format)
  }

  /**
   * Proxy to the remote `Processor`'s `execute` method
   */
  async execute (thing: string | object | Thing, format: string= 'application/ld+json'): Promise<Thing> {
    return this.call<Thing>('execute', thing, format)
  }

  /**
   * Call a method of a remote `Processor`.
   *
   * @param method The name of the method
   * @param args Any method arguments
   */
  async call<Type> (method: string, ...args: Array<any>): Promise<Type> {
    // Create a new request
    for (let index in args) {
      const arg = args[index]
      if (typeof arg === 'object' && arg.type) {
        args[index] = Object.assign({}, arg, { type: arg.type })
      }
    }
    const request = new JsonRpcRequest(method, args)
    // ... and a promise that resolves to the resul
    const promise = new Promise<Type>((resolve, reject) => {
      this.requests[request.id] = (response: JsonRpcResponse) => {
        // Fails if the response has an error
        if (response.error) return reject(new Error(response.error.message))
        // If the result is a `Thing` then we want to import it
        const result = response.result
        if (typeof result === 'object' && result.type) {
          const thing = processor.importObject(response.result)
          // @ts-ignore
          resolve(thing)
        } else {
          resolve(result)
        }
      }
    })
    // Now, we're ready to send the request
    this.send(JSON.stringify(request))
    return promise
  }

  /**
   * Send a request to the server.
   *
   * This method must be overriden by derived client classes to
   * send the request over the transport used by that class.
   *
   * @param request The JSON-RPC request as a string
   */
  abstract send (request: string): void

  /**
   * Receive a request from the server.
   *
   * Uses the `id` of the response to match it to the corresponding
   * request and resolve it's promise.
   *
   * @param response The JSON-RPC response as a string
   */
  recieve (response: string) {
    const responseObj = JSON.parse(response)
    if (!responseObj.id) throw new Error(`Response is missing id: ${response}`)
    const resolve = this.requests[responseObj.id]
    if (!resolve) throw new Error(`No request found for response with id: ${responseObj.id}`)
    resolve(responseObj)
  }
}
