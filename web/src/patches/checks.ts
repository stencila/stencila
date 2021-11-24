/**
 * A module with functions for checking for consistency between patch `Operation`
 * and the current state of the document.
 *
 * These checks are used liberally throughout the `patches` module with the rationale
 * that any inconsistency should trigger a "panic" to reset the state of the document.
 */

import { Slot } from '@stencila/stencila'

/**
 * Panic if there is a conflict between a `Patch` and the current DOM.
 *
 * This module make liberal use of assertions of consistency between `Operation`s
 * and the current DOM with the view that if there is any inconsistency detected then
 * it is best to simply exit the `applyPatch` function early and reload the page.
 *
 * This should only happen if there (a) the client has missed a `Patch`
 * such that the state of the DOM is out of sync with the server-side document, or
 * (b) if there is a bug in the following code. Hopefully testing rules out (b).
 *
 * Reloads the document to get a new DOM state and then throws an exception for
 * early exit from the calling function.
 */
export function panic(message: string): Error {
  // TODO reload the document
  return new Error(message)
}

/**
 * Assert that a condition is true and panic if it is not.
 */
export function assert(condition: boolean, message: string): void {
  if (!condition) {
    throw panic(message)
  }
}

/**
 * Is a slot a name (string) variant?
 */
export function isName(slot: Slot | undefined): slot is string {
  return typeof slot === 'string'
}

/**
 * Assert that a slot is a name (string)  variant.
 */
export function assertName(slot: Slot | undefined): asserts slot is string {
  assert(isName(slot), 'Expected string slot')
}

/**
 * Is a slot an index (integer) variant?
 */
export function isIndex(slot: Slot | undefined): slot is number {
  return typeof slot === 'number'
}

/**
 * Assert that a slot is an index (integer) variant.
 */
export function assertIndex(slot: Slot | undefined): asserts slot is number {
  assert(isIndex(slot), 'Expected number slot')
}

/**
 * Is a DOM node an element?
 */
export function isElement(node: Node | undefined): node is Element {
  return node?.nodeType === Node.ELEMENT_NODE
}

/**
 * Assert that a DOM node is an element
 */
export function assertElement(node: Node | undefined): asserts node is Element {
  assert(isElement(node), 'Expected element node')
}

/**
 * Is a DOM node an attribute?
 */
export function isAttr(node: Node | undefined): node is Attr {
  return node?.nodeType === Node.ATTRIBUTE_NODE
}

/**
 * Is a DOM node a text node?
 */
export function isText(node: Node | undefined): node is Text {
  return node?.nodeType === Node.TEXT_NODE
}

export type JsonValue =
  | string
  | number
  | boolean
  | null
  | JsonValue[]
  | { [key: string]: JsonValue }
export type JsonArray = JsonValue[]
export type JsonObject = { [key: string]: JsonValue }

/**
 * Assert that a JSON value is defined.
 */
export function assertDefined(value: unknown): asserts value is JsonValue {
  assert(value !== undefined, 'Expected value to be defined')
}

/**
 * Is a JSON value a number?
 */
export function isNumber(value: unknown): value is number {
  return typeof value === 'number'
}

/**
 * Assert that a JSON value is a number
 */
export function assertNumber(value: unknown): asserts value is number {
  assert(isNumber(value), `Expected a number, got a ${typeof value}`)
}

/**
 * Is a JSON value a string?
 */
export function isString(value: unknown): value is string {
  return typeof value === 'string'
}

/**
 * Assert that a JSON value is a string
 */
export function assertString(value: unknown): asserts value is string {
  assert(isString(value), `Expected a string, got a ${typeof value}`)
}

/**
 * Is a JSON value an array?
 */
export function isArray(value: unknown): value is JsonArray {
  return Array.isArray(value)
}

/**
 * Is a JSON value an object?
 */
export function isObject(value: unknown): value is JsonObject {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

/**
 * Assert that a JSON value is an array
 */
export function assertArray(value: unknown): asserts value is JsonArray {
  assert(isArray(value), `Expected an array, got a ${typeof value}`)
}

/**
 * Assert that a JSON value is an array or object
 */
export function assertArrayOrObject(
  value: JsonValue
): asserts value is JsonArray | JsonObject {
  assert(
    isArray(value) || isObject(value),
    `Expected an array or object, got a ${typeof value}`
  )
}
