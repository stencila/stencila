import * as documents from './documents'
import * as plugins from './plugins'
import * as projects from './projects'
import { Error } from './types'
const addon = require('../index.node')

type AnyFunction = (...args: any[]) => any

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
export function dispatchFn<F extends AnyFunction>(
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

/**
 * Utility function to wrap other functions with the `dispatchFn` helper.
 */
const wrap =
  <F extends AnyFunction>(fn: F) =>
  (...args: Parameters<F>): Result<ReturnType<F>> => {
    return dispatchFn(() => fn.apply(null, args))
  }

/**
 * Various library functions wrapped in the `dispatchFn` helper.
 * All function arguments are the same as the original, but the return types are
 * of type `Result` for a standardized usage by the client.
 *
 * TODO: See if type safety can be maintained while iterating over the imported objects.
 */
export const dispatch = {
  documents: {
    schemas: wrap(documents.schemas),
    list: wrap(documents.list),
    create: wrap(documents.create),
    open: wrap(documents.open),
    get: wrap(documents.get),
    alter: wrap(documents.alter),
    read: wrap(documents.read),
    write: wrap(documents.write),
    writeAs: wrap(documents.writeAs),
    dump: wrap(documents.dump),
    load: wrap(documents.load),
    subscribe: wrap(documents.subscribe),
    unsubscribe: wrap(documents.unsubscribe),
    close: wrap(documents.close),
  },
  plugins: {
    schema: wrap(plugins.schema),
    list: wrap(plugins.list),
    install: wrap(plugins.install),
    uninstall: wrap(plugins.uninstall),
    upgrade: wrap(plugins.upgrade),
    refresh: wrap(plugins.refresh),
  },
  projects: {
    schemas: wrap(projects.schemas),
    list: wrap(projects.list),
    open: wrap(projects.open),
    write: wrap(projects.write),
    subscribe: wrap(projects.subscribe),
    unsubscribe: wrap(projects.unsubscribe),
    close: wrap(projects.close),
  },
}
