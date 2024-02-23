import { DocumentId } from '../types'
import { Secret } from '../types/api'

export type APIError = {
  status: 'error'
  error: Error
}

export type APISuccess = {
  status: 'success'
}

export type APISuccessResponse<Payload> = APISuccess & {
  response: Payload
}

/**
 * Defines the discriminated payload that a response from a method
 * could provide.
 *
 * The "Payload" is the response that returns from the server when successful
 */
export type APIResponse<Payload> = Promise<
  APIError | APISuccessResponse<Payload>
>

/**
 * Defines the response we should return when we are not expecting responses
 * back from the server
 */
export type APIRequest = Promise<APIError | APISuccess>

/**
 * Params that are sent to request functions
 */
type RequestParams = {
  url: string
  init?: RequestInit
  code?: number
}

/**
 * Params that are sent to handle request calls.
 */
type HandleRequestParams<ResponsePayload> = RequestParams & {
  callback: (response?: Response) => ResponsePayload
}

/**
 * A function that fetches a url (plus optional fetch requestInit) and
 * returns a promise of a discriminated union - either an error or the payload.
 *
 * @template ResponsePayload The payload to expect (can be undefined)
 * @param {AcceptableURLs} url
 * @param {RequestInit} [init]
 * @return {*}  {APIResponse<ResponsePayload>}
 */
export const requestApiResponseAsJson = async <ResponsePayload,>({
  url,
  init,
  code,
}: RequestParams): APIResponse<ResponsePayload> => {
  return handleRequest({
    url,
    init,
    code,
    callback: async (response) => {
      try {
        const result = (await response.json()) as ResponsePayload
        return {
          status: 'success' as const,
          response: result,
        }
      } catch (e) {
        const error =
          e instanceof Error ? e : new Error('unknown error occurred')
        return {
          status: 'error' as const,
          error,
        }
      }
    },
  })
}

/**
 * Fetches an endpoint that simply returns a response, does not return a payload
 *
 * @param url
 * @param init
 * @returns {APIRequest}
 */
export const requestApi = async ({
  url,
  init,
  code,
}: RequestParams): APIRequest => {
  return handleRequest({
    url,
    init,
    code,
    callback: () => ({
      status: 'success' as const,
    }),
  })
}

/**
 * Handles a fetch request & provides the correct response. Handles error
 * handling as well. The response returned will be a discriminated union of
 * either an error or the correct payload.
 *
 * @template ResponsePayload
 * @param {HandleRequestParams<ResponsePayload>} {
 *   url,
 *   init,
 *   code,
 *   callback,
 * }
 * @return {*}
 */
const handleRequest = async <ResponsePayload,>({
  url,
  init,
  code,
  callback,
}: HandleRequestParams<ResponsePayload>) => {
  try {
    const response = await fetch(url, init)

    if ((code && response.status !== code) || !response.ok) {
      return {
        status: 'error' as const,
        error: new Error(response.statusText),
      }
    }

    return await callback(response)
  } catch (e) {
    const error = e instanceof Error ? e : new Error('unknown error occurred')

    return {
      status: 'error' as const,
      error,
    }
  }
}

/**
 * This class is a wrapper for all internal API requests.
 *
 * @export
 * @class RestAPIClient
 */
export class RestAPIClient {
  /**
   * Open the document at the supplied _path_
   *
   * @static
   * @param {string} path
   * @return {*}
   * @memberof RestAPIClient
   */
  static async openDocument(path: string) {
    return requestApiResponseAsJson<{ id: DocumentId }>({
      url: `/~open/${path}`,
    })
  }

  /**
   * close a document on the server
   *
   * @param {DocumentId} docId
   * @memberof RestAPIClient
   */
  static async closeDocument(docId: DocumentId) {
    await requestApi({ url: `/~close/${docId}`, init: { method: 'POST' } })
  }

  /**
   * Retrieve a list of configured secrets for the application.
   *
   * @static
   * @return {*}
   * @memberof RestAPIClient
   */
  static async listSecrets() {
    return requestApiResponseAsJson<Secret[]>({ url: '/~secrets' })
  }

  /**
   * Sets the value of a secret in the API.
   *
   * @static
   * @param {string} name
   * @param {string} value
   * @return {*}
   * @memberof RestAPIClient
   */
  static async setSecret(name: string, value: string) {
    return requestApi({
      url: `/~secrets/${name}`,
      init: {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain', body: value },
      },
    })
  }

  /**
   * Delete the secret defined by _name_
   *
   * @static
   * @param {string} name
   * @return {*}
   * @memberof RestAPIClient
   */
  static async deleteSecret(name: string) {
    return requestApi({
      url: `/~secrets/${name}`,
      init: {
        method: 'DELETE',
      },
    })
  }
}
