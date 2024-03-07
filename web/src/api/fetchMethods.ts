/**
 * Provides a number of methods to fetch and handle responses from the
 * server API.
 */

/**
 * Params that are sent to request functions
 * - url: API endpoint
 * - identifier: id to assist synchronisation of multiple promises
 * - init: fetch params
 * - code: expected "success" status code - will default to 200.
 */
type RequestParams = {
  url: string
  identifier?: string
  init?: RequestInit
  code?: number
}

export type APIError = {
  status: 'error'
  error: Error
  identifier?: string
}

export type APISuccess = {
  status: 'success'
  identifier?: string
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
 * Params that are sent to handle request calls.
 */
type HandleRequestParams<ResponsePayload> = RequestParams & {
  callback: (response?: Response) => ResponsePayload
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
  identifier,
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
 * Provides static methods to access our API.
 *
 * @export
 * @class APIAccess
 */
export class APIAccess {
  /**
   * Fetches an endpoint that simply returns a response, does not return a
   * payload - e.g. a POST with no response payload.
   *
   * @static
   * @param {RequestParams} {
   *     url,
   *     init,
   *     identifier,
   *     code,
   *   }
   * @return {*}  {APIRequest}
   * @memberof APIAccess
   */
  static async requestApi({
    url,
    init,
    identifier,
    code,
  }: RequestParams): APIRequest {
    return handleRequest({
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
   * A function that fetches a url (plus optional fetch requestInit) and returns
   * a promise of a discriminated union - either an error or the payload.
   *
   * @static
   * @template ResponsePayload The payload to expect (can be undefined)
   * @param {RequestParams} {
   *     url,
   *     init,
   *     identifier,
   *     code,
   *   }
   * @return {*}  {APIResponse<ResponsePayload>}
   * @memberof APIAccess
   */
  static async requestApiResponseAsJson<ResponsePayload>({
    url,
    init,
    identifier,
    code,
  }: RequestParams): APIResponse<ResponsePayload> {
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
}
