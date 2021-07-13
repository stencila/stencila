import { EntityId } from '@reduxjs/toolkit'
import type { Result, ResultFailure, ResultSuccess } from 'stencila'
import { CHANNEL } from '../preload/channels'
import { UnprotectedStoreKeys } from '../preload/stores'
import type { JSONValue } from '../preload/types'

/**
 * Custom Error instance thrown by `unwrapOrThrow`.
 * Allows for matching against this error type, and having custom handler logic.
 */
export class RPCError extends Error {
  public errors: ResultFailure['errors']

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
const unwrapOrThrow = <V>(result: Result<V>): ResultSuccess<V> => {
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
  app: {
    utils: {
      openLinkInBrowser: (url: string) =>
        window.api
          .invoke(CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER, url)
          .then(unwrapOrThrow),
    },
  },
  config: {
    global: {
      getAll: () => window.api.invoke(CHANNEL.CONFIG_READ).then(unwrapOrThrow),
    },
    ui: {
      getAll: () =>
        window.api.invoke(CHANNEL.CONFIG_APP_READ).then(unwrapOrThrow),
      get: (key: UnprotectedStoreKeys) =>
        window.api
          .invoke(CHANNEL.CONFIG_APP_GET, key)
          .then((res) => unwrapOrThrow(res as Result<JSONValue>)),
      set: ({ key, value }: { key: UnprotectedStoreKeys; value: JSONValue }) =>
        window.api
          .invoke(CHANNEL.CONFIG_APP_SET, {
            key,
            value,
          })
          .then(unwrapOrThrow),
    },
  },
  onboarding: {
    open: () =>
      window.api.invoke(CHANNEL.ONBOARDING_WINDOW_OPEN).then(unwrapOrThrow),
    close: () =>
      window.api.invoke(CHANNEL.ONBOARDING_WINDOW_CLOSE).then(unwrapOrThrow),
  },
  launcher: {
    open: () =>
      window.api.invoke(CHANNEL.LAUNCHER_WINDOW_OPEN).then(unwrapOrThrow),
    close: () =>
      window.api.invoke(CHANNEL.LAUNCHER_WINDOW_CLOSE).then(unwrapOrThrow),
  },
  documents: {
    open: (path: string, format?: string) =>
      window.api
        .invoke(CHANNEL.DOCUMENTS_OPEN, path, format)
        .then((d) => d)
        .then(unwrapOrThrow),
    contents: (docId: EntityId) =>
      window.api
        .invoke(CHANNEL.DOCUMENTS_DUMP, docId.toString())
        .then((r) => r)
        .then(unwrapOrThrow),
    preview: (docId: EntityId) =>
      window.api
        .invoke(CHANNEL.DOCUMENTS_PREVIEW, docId.toString())
        .then(unwrapOrThrow),
    unsubscribe: ({
      documentId,
      topics,
    }: {
      documentId: EntityId
      topics: string[]
    }) =>
      window.api
        .invoke(CHANNEL.DOCUMENTS_UNSUBSCRIBE, documentId.toString(), topics)
        .then(unwrapOrThrow),
    write: ({
      documentId,
      content,
    }: {
      documentId: EntityId
      content: string
    }) =>
      window.api
        .invoke(CHANNEL.DOCUMENTS_WRITE, documentId.toString(), content)
        .then(unwrapOrThrow),
  },
  projects: {
    open: (path: string) =>
      window.api.invoke(CHANNEL.PROJECTS_WINDOW_OPEN, path).then(unwrapOrThrow),
    contents: (path: string) =>
      window.api.invoke(CHANNEL.PROJECTS_OPEN, path).then(unwrapOrThrow),
    openUsingPicker: () =>
      window.api.invoke(CHANNEL.PROJECTS_OPEN_FROM_FILE_PICKER).then(unwrapOrThrow),
  },
  plugins: {
    install: (name: string) =>
      window.api.invoke(CHANNEL.PLUGINS_INSTALL, name).then(unwrapOrThrow),
    uninstall: (name: string) =>
      window.api.invoke(CHANNEL.PLUGINS_UNINSTALL, name).then(unwrapOrThrow),
    list: () =>
      window.api.invoke(CHANNEL.PLUGINS_LIST).then(unwrapOrThrow),
    refresh: (plugins: string[] = []) =>
      window.api.invoke(CHANNEL.PLUGINS_REFRESH, plugins).then(unwrapOrThrow),
  },
}
