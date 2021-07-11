import { Error } from './types'
const addon = require('../index.node')

type AnyFunction = (...args: unknown[]) => any

/**
 * A result of a call to one of the function in this package
 *
 * The value type is `unknown` here, but inferred by the client methods.
 */
export type ResultSuccess<V = unknown> = {
  ok: true
  value: V
  errors: Error[]
}

export type ResultFailure = {
  ok: false
  errors: Error[]
}

export type Result<V = unknown> = ResultSuccess<V> | ResultFailure

/**
 * Dispatch a call
 *
 * Catches any errors during the call, parses them into an object and returns
 * them as part of the `Result`.
 */
export function dispatch<F extends AnyFunction>(
  callback: F
): Result<ReturnType<F>> {
  addon.errorsStart()

  let ok = true
  let value
  let errors: Error[] = []

  try {
    value = callback()
  } catch (err) {
    ok = false
    try {
      errors = [JSON.parse(err.message)]
    } catch {
      errors = [
        {
          type: 'Unspecified',
          message: err.message,
        },
      ]
    }
  }

  try {
    const sidebandErrors = JSON.parse(addon.errorsStop())
    errors = [...sidebandErrors, ...errors]
  } catch {}

  if (ok) {
    return { ok, value, errors }
  } else {
    return { ok, errors }
  }
}
