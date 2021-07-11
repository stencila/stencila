import { Result, ResultFailure } from 'stencila'
import { CHANNEL } from '../preload/channels'

/**
 * Custom Error instance thrown by `unwrapOrThrow`.
 * Allows for matching against this error type, and having custom handler logic.
 */
export class RPCError extends Error {
  errors: ResultFailure['errors']

  constructor(
    errors: ResultFailure['errors'],
    ...params: (string | undefined)[]
  ) {
    // Pass remaining arguments (including vendor specific ones) to parent constructor
    super(...params)

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace) {
      Error.captureStackTrace(this, RPCError)
    }

    this.name = 'RPCError'
    this.errors = errors
  }
}

/**
 * Takes the result of an RPC call, and refines the type to a success object.
 * In case of a failed execution, throws an error.
 * This allows for a `Promise`-like usage of the RPC calls.
 *
 * @example
 * window.api
 *  .invoke(CHANNEL.DOCUMENTS_OPEN, path, format)
 *  .then(unwrapOrThrow)
 *  .then(({value}) => value.id)
 *  .catch((err) => {
 *    if (isRPCError(err)) {
 *      // do something
 *    } else {
 *       // Generic error handler
 *    }
 *  })
 */
const unwrapOrThrow = <R>(result: Result<R>) => {
  if (result.ok) {
    return result
  } else {
    throw new RPCError(result.errors)
  }
}

export const isRPCError = (error: Error): error is RPCError => {
  return error instanceof RPCError
}

// -----------------------------------------------------------------------------

export const client = {
  documents: {
    open: (path: string, format?: string) =>
      window.api
        .invoke(CHANNEL.DOCUMENTS_OPEN, path, format)
        .then(unwrapOrThrow),
  },
}
