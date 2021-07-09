import { documents } from '.'
import { Error } from './types'

const addon = require('../index.node')

// A call to one of the functions in this package
//
// For consistency, uses the property names `method` and `params`
// as in a JSON-RPC request
export interface Call {
  method: string
  params: unknown[]
}

// A result of a call to one of the function in this package
//
// The value type is `unknown` here, but known by the client
// methods.
export interface Result {
  value?: unknown
  errors?: Error[]
}

// Dispatch a call
//
// Catches any errors during the call, parses them into an object
// and returns them as part of the `Result`.
export function dispatch(call: Call): Result {
  const { method, params } = call
  const func = resolve(method)

  addon.errorsStart()

  let value
  let errors: Error[] = []
  try {
    // @ts-ignore
    value = func.apply(null, params)
  } catch (err) {
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

  return { value, errors }
}

// Resolve a function from the `method` string of a call
function resolve(method: string) {
  switch (method) {
    case 'documentsOpen':
      return documents.open
  }
}
