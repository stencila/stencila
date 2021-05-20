// Node.js bindings for ../../rust/src/projects.rs, see there for more documentation.

import { JSONSchema7 } from 'json-schema'
import { fromJSON } from './prelude'
import { subscribe, Subscriber } from './pubsub'
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
 * @param path Path to the document's file
 * @param subscriber A subscriber function that will receive published
 *                   events for the document
 * @return A document
 */
export function open(
  path: string,
  subscriber?: (topic: string, event: DocumentEvent) => unknown
): Document {
  const document = fromJSON<Document>(addon.documentsOpen(path))
  if (subscriber !== undefined)
    subscribe(`document:${document.path}`, subscriber as Subscriber)
  return document
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
