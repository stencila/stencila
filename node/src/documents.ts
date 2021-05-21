// Node.js bindings for ../../rust/src/projects.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON } from './prelude'
import * as pubsub from './pubsub'
import { Document, DocumentEvent } from './types'

const addon = require('../index.node')

/**
 * Get the JSON Schemas for a document module
 *
 * @returns An array of JSON Schema v7 objects describing the properties of
 *          types in this module
 */
export function schemas(): JSONSchema7 {
  return fromJSON<JSONSchema7>(addon.documentsSchema())
}

/**
 * List documents that are currently open
 *
 * @returns An array of documents
 */
export function list(): Document[] {
  return fromJSON<Document[]>(addon.documentsList())
}

/**
 * Open a document
 *
 * If you want the document's content you need to `open(<path>)` it
 * and then `subscribe(<path>, ['content'], (topic, event) => ...)` it.
 *
 * @param path Path to the document's file
 * @return A document
 */
export function open(path: string): Document {
  return fromJSON<Document>(addon.documentsOpen(path))
}

/**
 * Close a document
 *
 * @param path Path to the document's file
 */
export function close(path: string): void {
  addon.documentsClose(path)
}

/**
 * Get a document
 *
 * Currently, the same as open but may change.
 */
export const get = open

/**
 * Subscribe to one or more of the document's topics
 *
 * @param path Path to the document's file
 * @param topic See docs for `Document#subscriptions` for valid values
 * @param subscriber A subscriber function that will receive published
 *                   events for the document topic/s
 */
export function subscribe(
  path: string,
  topics: string[],
  subscriber: (topic: string, event: DocumentEvent) => unknown
): void {
  for (const topic of topics) {
    addon.documentsSubscribe(path, topic)
    pubsub.subscribe(
      `documents:${path}:${topic}`,
      subscriber as pubsub.Subscriber
    )
  }
}

/**
 * Unsubscribe from one or more of the document's topics
 *
 * @param path Path to the document's file
 * @param subscriber A subscriber function that will receive published
 *                   events for the document topic/s
 */
export function unsubscribe(path: string, topics: string[]): void {
  for (const topic of topics) {
    addon.documentsUnsubscribe(path, topic)
    pubsub.unsubscribe(`documents:${path}:${topic}`)
  }
}

/**
 * Read a document from the file system.
 *
 * @param path Path to the document's file
 */
export function read(path: string): string {
  return addon.documentsRead(path)
}

/**
 * Dump the current content of the document
 * without reading it from the file system.
 * The inverse of `load()`.
 *
 * @param path Path to the document's file
 */
export function dump(path: string): string {
  return addon.documentsDump(path)
}

/**
 * Load content into the document without writing it
 * to the file system. The inverse of `dump()`.
 *
 *
 * @param path Path to the document's file
 */
export function load(path: string, content: string): void {
  return addon.documentsLoad(path, content)
}

/**
 * Write the content of the document to the file system.
 *
 * @param path Path to the document's file
 */
export function write(path: string, content: string): string {
  return addon.documentsWrite(path, content)
}
