import { Address, Slot } from '@stencila/stencila'
import GraphemeSplitter from 'grapheme-splitter'

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
export function isString(slot: Slot): slot is string {
  return typeof slot === 'string'
}

/**
 * Assert that a slot is a string variant.
 */
export function assertString(slot: Slot): asserts slot is string {
  assert(isString(slot), 'Expected string slot')
}

/**
 * Is a slot a number variant?
 */
export function isNumber(slot: Slot): slot is number {
  return typeof slot === 'number'
}

/**
 * Assert that a slot is a number variant.
 */
export function assertNumber(slot: Slot): asserts slot is number {
  assert(isNumber(slot), 'Expected number slot')
}

/**
 * Is a DOM node an element?
 */
export function isElement(node: Node): node is Element {
  return node.nodeType === Node.ELEMENT_NODE
}

/**
 * Assert that a DOM node is an element
 */
export function assertElement(node: Node): asserts node is Element {
  assert(isElement(node), 'Expected element node')
}

/**
 * Is a DOM node a text node?
 */
export function isText(node: Node): node is Text {
  return node.nodeType === Node.TEXT_NODE
}

/**
 * Resolve the root node.
 *
 * Addresses are relative to the root, so it is always necessary
 * to resolve this first.
 *
 * Panics if unable to find the `[slot="root"]` node in the
 * body of the DOM document.
 */
export function resolveRoot(): Element {
  const root = document.body.querySelector('[slot="root"]')
  if (root === null) throw panic('Unable to resolve root node')
  return root
}

/**
 * Resolve a slot in a parent DOM node.
 *
 * Note that the `parent` must be an `Element` but that the returned
 * node may be an `Element` or a `Text` DOM node.
 */
export function resolveSlot(parent: Element, slot: Slot): Element | Text {
  if (isString(slot)) {
    // Select the first child element with the slot.
    // This could perhaps be loosened, by removing `:scope` so the first descendent is selected.
    assertElement(parent)
    const next: Element | null = parent.querySelector(
      `:scope > [slot="${slot}"]`
    )
    if (next === null) {
      // The `content` property can be is an "implicit" slot; if it is not
      // present then just return the parent.
      if (slot === 'content') return parent
      else throw panic(`Unable to resolve slot '${slot}''`)
    }
    return next
  } else {
    // Select the child at the slot index.
    const next: ChildNode | undefined = parent.childNodes[slot]
    if (next === undefined) {
      throw panic(
        `Unable to get slot '${slot}' from element of with ${parent.childNodes.length} children`
      )
    } else if (isElement(next) || isText(next)) {
      return next
    } else {
      throw panic('Unexpected node type')
    }
  }
}

/**
 * Resolve the parent of the node at the address.
 *
 * Returns the parent node and the target node's slot.
 * This is used for `Add` and `Replace` operations where we need
 * the node on which to perform the action and the terminal slot
 * refers to the location within that node to add or replace.
 */
export function resolveParent(address: Address): [Element | Text, Slot] {
  let parent: Element | Text = resolveRoot()

  for (const slot of address.slice(0, -1)) {
    assertElement(parent)
    parent = resolveSlot(parent, slot)
  }

  const slot = address[address.length - 1]
  if (slot === undefined) throw panic('Address is too short')

  return [parent, slot]
}

/**
 * Resolve the node at the address.
 */
export function resolveNode(address: Address): Element | Text {
  let node: Element | Text = resolveRoot()

  for (const slot of address) {
    assertElement(node)
    node = resolveSlot(node, slot)
  }

  return node
}

/**
 * Create a DOM fragment from a HTML string
 */
export function createFragment(html: string): DocumentFragment {
  return document.createRange().createContextualFragment(html)
}

const GRAPHEME_SPLITTER = new GraphemeSplitter()

/**
 * Split a string into Unicode graphemes
 */
export function toGraphemes(text: string): string[] {
  return GRAPHEME_SPLITTER.splitGraphemes(text)
}
