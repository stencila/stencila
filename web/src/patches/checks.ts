/**
 * A module with functions for checking for consistency between patch `Operation`
 * and the current state of the document.
 *
 * These checks are used liberally throughout the `patches` module with the rationale
 * that any inconsistency should trigger a "panic" to reset the state of the document.
 */

import { Slot } from '@stencila/stencila'

/**
 * Panic if there is a conflict between a `DomPatch` and the current DOM.
 *
 * This module make liberal use of assertions of consistency between `DomOperation`s
 * and the current DOM with the view that if there is any inconsistency detected then
 * it is best to simply exit the `applyPatch` function early and reload the page.
 *
 * This should only happen if there (a) the client has missed a `DomPatch`
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
 * Is a slot a string variant?
 */
export function isString(slot: Slot | undefined): slot is string {
  return typeof slot === 'string'
}

/**
 * Assert that a slot is a string variant.
 */
export function assertString(slot: Slot | undefined): asserts slot is string {
  assert(isString(slot), 'Expected string slot')
}

/**
 * Is a slot a number variant?
 */
export function isNumber(slot: Slot | undefined): slot is number {
  return typeof slot === 'number'
}

/**
 * Assert that a slot is a number variant.
 */
export function assertNumber(slot: Slot | undefined): asserts slot is number {
  assert(isNumber(slot), 'Expected number slot')
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
export function assertElement(node: Node): asserts node is Element {
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
