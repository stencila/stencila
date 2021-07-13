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
        window.api.invoke(CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER, url),
    },
  },
  config: {
    global: {
      getAll: () => window.api.invoke(CHANNEL.READ_CONFIG).then(unwrapOrThrow),
    },
    ui: {
      getAll: () => window.api.invoke(CHANNEL.READ_APP_CONFIG),
      get: (key: UnprotectedStoreKeys) =>
        window.api.invoke(CHANNEL.GET_APP_CONFIG, key).then(unwrapOrThrow),
      set: ({ key, value }: { key: UnprotectedStoreKeys; value: JSONValue }) =>
        window.api.invoke(CHANNEL.SET_APP_CONFIG, {
          key,
          value,
        }),
    },
  },
  onboarding: {
    open: () => window.api.invoke(CHANNEL.OPEN_ONBOARDING_WINDOW),
    close: () => window.api.invoke(CHANNEL.CLOSE_ONBOARDING_WINDOW),
  },
  launcher: {
    open: () => window.api.invoke(CHANNEL.OPEN_LAUNCHER_WINDOW),
    close: () => window.api.invoke(CHANNEL.CLOSE_LAUNCHER_WINDOW),
  },
  documents: {
    open: (path: string, format?: string) =>
      window.api
        .invoke(CHANNEL.DOCUMENTS_OPEN, path, format)
        .then(unwrapOrThrow),
    contents: (docId: EntityId) =>
      window.api
        .invoke(CHANNEL.GET_DOCUMENT_CONTENTS, docId)
        .then(unwrapOrThrow),
    preview: (docId: EntityId) =>
      window.api
        .invoke(CHANNEL.GET_DOCUMENT_PREVIEW, docId)
        .then(unwrapOrThrow),
    unsubscribe: ({
      documentId,
      topics,
    }: {
      documentId: EntityId
      topics: string[]
    }) =>
      window.api
        .invoke(CHANNEL.UNSUBSCRIBE_DOCUMENT, { documentId, topics })
        .then(unwrapOrThrow),
    write: ({
      documentId,
      content,
    }: {
      documentId: EntityId
      content: string
    }) =>
      window.api
        .invoke(CHANNEL.SAVE_DOCUMENT, { documentId, content })
        .then(unwrapOrThrow),
  },
  projects: {
    open: (path: string) =>
      window.api.invoke(CHANNEL.OPEN_PROJECT, path).then(unwrapOrThrow),
    contents: (path: string) =>
      window.api.invoke(CHANNEL.GET_PROJECT_FILES, path).then(unwrapOrThrow),
    openUsingPicker: () => window.api.invoke(CHANNEL.SELECT_PROJECT_DIR),
  },
  plugins: {
    install: (name: string) =>
      window.api.invoke(CHANNEL.INSTALL_PLUGIN, name).then(unwrapOrThrow),
    uninstall: (name: string) =>
      window.api.invoke(CHANNEL.UNINSTALL_PLUGIN, name).then(unwrapOrThrow),
    list: () =>
      window.api.invoke(CHANNEL.LIST_AVAILABLE_PLUGINS).then(unwrapOrThrow),
    refresh: (plugins: string[] = []) =>
      window.api.invoke(CHANNEL.REFRESH_PLUGINS, plugins).then(unwrapOrThrow),
  },
}
