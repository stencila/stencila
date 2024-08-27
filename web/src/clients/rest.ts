import { DocumentId } from '../types'

import { DocumentCommand } from './commands'

/**
 * The parameters of a REST API request
 */
type RestRequestParams = {
  /**
   * The URL of the REST API endpoint
   */
  url: string

  /**
   * An identifier to assist synchronization of multiple promises
   */
  identifier?: string

  /**
   * Fetch parameters
   */
  init?: RequestInit

  /**
   * Expected status code of response (defaults to 200)
   */
  code?: number
}

/**
 * The type of a REST API error response
 */
type RestError = {
  status: 'error'
  error: Error
  identifier?: string
}

/**
 * The type of a successful REST API response when no payload is expected.
 */
type RestSuccess = {
  status: 'success'
  identifier?: string
}

/**
 * The type of a successful REST API response when a payload is expected.
 */
type RestSuccessWithPayload<T> = RestSuccess & {
  response: T
}

/**
 * The type of REST API response when a payload is not expected.
 */
type RestResponse = Promise<RestError | RestSuccess>

/**
 * The type of REST API response when a payload is expected.
 *
 * `T` is the type of the expected response payload.
 */
type RestResponseWithPayload<T> = Promise<RestError | RestSuccessWithPayload<T>>

/**
 * A client for the Stencila server's REST API
 *
 * This class exposes all method available in that API. See the
 * Rust `server` crate for which endpoints are available.
 */
export class RestClient {
  /**
   * Creates and sends a `fetch` request and handles the response
   *
   * Returns a discriminated union of the payload, or an error.
   * `T` is the type of the expected response payload.
   */
  static async makeRequest<T>({
    url,
    identifier,
    init,
    code,
    callback,
  }: RestRequestParams & {
    callback: (response?: Response) => T
  }) {
    try {
      const response = await fetch(url, init)

      if ((code && response.status !== code) || !response.ok) {
        return {
          status: 'error' as const,
          error: new Error(response.statusText),
          identifier,
        }
      }

      return await callback(response)
    } catch (e) {
      const error = e instanceof Error ? e : new Error('unknown error occurred')

      return {
        status: 'error' as const,
        error,
        identifier,
      }
    }
  }

  /**
   * Make a request with no expected response payload
   */
  static async request({
    url,
    init,
    identifier,
    code,
  }: RestRequestParams): RestResponse {
    return RestClient.makeRequest({
      url,
      init,
      identifier,
      code,
      callback: () => {
        return {
          status: 'success' as const,
          identifier,
        }
      },
    })
  }

  /**
   * Make a request with an expected response payload of a specific type
   */
  static async requestAs<T>({
    url,
    init,
    identifier,
    code,
  }: RestRequestParams): RestResponseWithPayload<T> {
    return RestClient.makeRequest({
      url,
      init,
      code,
      callback: async (response: Response) => {
        try {
          const result = (await response.json()) as T
          return {
            status: 'success' as const,
            response: result,
            identifier,
          }
        } catch (e) {
          const error =
            e instanceof Error ? e : new Error('unknown error occurred')
          return {
            status: 'error' as const,
            error,
            identifier,
          }
        }
      },
    })
  }

  /**
   * Open the document at the supplied `path`
   */
  static async openDocument(path: string) {
    return RestClient.requestAs<{ id: DocumentId }>({
      url: `/~documents/open/${path}`,
      identifier: path,
    })
  }

  /**
   * Close a document on the server with `docId`
   */
  static async closeDocument(docId: DocumentId) {
    await RestClient.request({
      url: `/~documents/${docId}/close`,
      identifier: docId,
      init: { method: 'POST' },
    })
  }

  /**
   * Send a command to a document
   */
  static async documentCommand(docId: DocumentId, command: DocumentCommand) {
    await RestClient.request({
      url: `/~documents/${docId}/command`,
      identifier: docId,
      init: {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(command),
      },
    })
  }
}
