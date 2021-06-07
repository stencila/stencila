// Node.js bindings for ../../rust/src/projects.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON } from './prelude'
import * as pubsub from './pubsub'
import { Document, DocumentEvent } from './types'

const addon = require('../index.node')

/**
 * Get the JSON Schemas associated with the `documents` module
 *
 * @returns An array of JSON Schema v7 objects
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
 * Create a new empty document, optionally specifying its format.
 *
 * @param format Format of the document
 * @return A document
 */
export function create(format?: string): Document {
  return fromJSON<Document>(addon.documentsCreate(format ?? ''))
}

/**
 * Open an existing document, optionally specifying its format.
 *
 * If you want the document's content you need to `open(<path>)` it
 * and then `subscribe(<path>, ['content'], (topic, event) => ...)` to it.
 *
 * @param path Path to the document's file
 * @param format Format of the document. If none will be inferred from
 *               the file extension.
 * @return A document
 */
export function open(path: string, format?: string): Document {
  return fromJSON<Document>(addon.documentsOpen(path, format ?? ''))
}

/**
 * Get a document
 *
 * @param id Id of the document
 */
export function get(id: string): Document {
  return fromJSON<Document>(addon.documentsGet(id))
}

/**
 * Read a document from the file system.
 *
 * @param id Id of the document
 */
export function read(id: string): string {
  return addon.documentsRead(id)
}

/**
 * Write the content of a document to the file system.
 *
 * @param id Id of the document
 */
export function write(id: string, content: string): string {
  return addon.documentsWrite(id, content)
}

/**
 * Dump the current content of a document
 * without reading it from the file system.
 * The inverse of `load()`.
 *
 * @param id Id of the document
 */
export function dump(id: string, format?: string): string {
  return addon.documentsDump(id, format ?? '')
}

/**
 * Load content into a document without writing it
 * to the file system. The inverse of `dump()`.
 *
 *
 * @param id Id of the document
 */
export function load(id: string, content: string): void {
  return addon.documentsLoad(id, content)
}

/**
 * Subscribe to one or more of a document's topics
 *
 * @param id Id of the document
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
 * Unsubscribe from one or more of a document's topics
 *
 * @param id Id of the document
 * @param subscriber A subscriber function that will receive published
 *                   events for the document topic/s
 */
export function unsubscribe(id: string, topics: string[]): void {
  for (const topic of topics) {
    addon.documentsUnsubscribe(id, topic)
    pubsub.unsubscribe(`documents:${id}:${topic}`)
  }
}

/**
 * Close a document
 *
 * @param id Id of the document
 */
export function close(id: string): void {
  addon.documentsClose(id)
}
