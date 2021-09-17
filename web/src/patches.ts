/**
 * This module implements DOM-based analogues of several functions in `../rust/src/patches.rs`.
 *
 * Compared to the Rust functions which apply a `Patch` to a Stencila `Node` (e.g. an `Article`
 * or `String`), these functions apply a `DomPatch` to a DOM `Node` (e.g. an `Element`, or `Text`).
 */

import {
  Address,
  DomOperation,
  DomOperationAdd,
  DomOperationMove,
  DomOperationRemove,
  DomOperationReplace,
  DomOperationTransform,
  DomPatch,
  Slot,
} from '@stencila/stencila'

/**
 * Apply a `DomPatch` to the `root` node of the current document
 */
export function applyPatch(patch: DomPatch): void {
  for (const op of patch) {
    applyOp(op)
  }
}

/**
 * Apply a `DomOperation` to the `root` node
 *
 * Uses the `type` discriminant to dispatch to specific functions for
 * each operation variant.
 */
export function applyOp(op: DomOperation): void {
  switch (op.type) {
    case 'Add':
      return applyAdd(op)
    case 'Remove':
      return applyRemove(op)
    case 'Replace':
      return applyReplace(op)
    case 'Move':
      return applyMove(op)
    case 'Transform':
      return applyTransform(op)
  }
}

/**
 * Panic because if there is a conflict between a `DomOperation` and the
 * current DOM.
 *
 * This should only happen if there (a) the client has missed a `DomOperation`
 * such that the state of the DOM is out of sync with the server-side document, or
 * (b) if there is a bug in the following code. Hopefully testing rules out (b).
 *
 * Reloads the page to get a new DOM state and then throws an exception for
 * early exit from the calling function
 */
function panic(message: string): void {
  window.location.reload()
  throw new Error(message)
}

/**
 * Assert that a condition is true and panic if it is not.
 */
function assert(condition: boolean, message: string): void {
  if (!condition) {
    panic(message)
  }
}

/**
 * Is a slot a string variant?
 */
function isString(slot: Slot): slot is string {
  return typeof slot === 'string'
}

/**
 * Assert that a slot is a string variant.
 */
function _assertString(slot: Slot): asserts slot is string {
  assert(isString(slot), 'Expected string slot')
}

/**
 * Is a slot a number variant?
 */
function isNumber(slot: Slot): slot is number {
  return typeof slot === 'number'
}

/**
 * Assert that a slot is a number variant.
 */
function assertNumber(slot: Slot): asserts slot is number {
  assert(isNumber(slot), 'Expected number slot')
}

/**
 * Is a DOM node an element?
 */
function isElement(node: Node): node is Element {
  return node.nodeType === Node.ELEMENT_NODE
}

/**
 * Assert that a DOM node is an element
 */
function assertElement(node: Node): asserts node is Element {
  assert(isElement(node), 'Expected element node')
}

/**
 * Is a DOM node a text node?
 */
function isText(node: Node): node is Text {
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
function resolveRoot(): Element {
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
function resolveSlot(parent: Element, slot: Slot): Element | Text {
  if (isString(slot)) {
    // Select the first child element with the slot.
    // This could perhaps be loosened, by removing `:scope` so the first descendent is selected.
    assertElement(parent)
    const next: Element | null = parent.querySelector(
      `:scope > [slot="${slot}"]`
    )
    if (next === null) {
      if (slot !== 'content') throw panic(`Unable to resolve slot ${slot}`)
      else return parent
    }
    return next
  } else {
    // Select the child at the slot index.
    const next: ChildNode | undefined = parent.childNodes[slot]
    if (next === undefined) {
      throw panic(
        `Unable to get slot ${slot} from element of with ${parent.childNodes.length} children`
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
function resolveParent(address: Address): [Element | Text, Slot] {
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
function resolveNode(address: Address): Element | Text {
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
function createFragment(html: string): DocumentFragment {
  return document.createRange().createContextualFragment(html)
}

/**
 * Apply an add operation
 */
export function applyAdd(op: DomOperationAdd): void {
  const { address, html } = op
  const [parent, slot] = resolveParent(address)

  if (isElement(parent)) {
    const fragment = createFragment(html)
    if (isString(slot)) {
      // If the slot is a string then we're adding a value to an `Option`
      // property of a `struct` (one that was previous `None`) so can just append the fragment.
      parent.appendChild(fragment)
    } else {
      // If the slot is number then we're inserting an item to a `Vec` in which
      // case we need to make sure we're inserting at in correct position.
      const children = parent.childNodes
      if (children.length === 0) {
        assert(
          slot === 0,
          'Attempting to insert at non-zero slot in element with no children'
        )
        parent.appendChild(fragment)
      } else {
        const sibling = parent.childNodes[slot]
        if (sibling === undefined)
          throw panic(
            `Unexpected add slot ${slot} for element with ${children.length} children`
          )
        parent.insertBefore(fragment, sibling)
      }
    }
  } else {
    // Inserting into a string
    assertNumber(slot)
    const text = parent.textContent ?? ''
    assert(
      slot >= 0 && slot <= text.length,
      `Unexpected add slot ${slot} for text node of length ${text.length}`
    )
    parent.textContent = text.slice(0, slot) + html + text.slice(slot)
  }
}

/**
 * Apply a remove operation
 */
export function applyRemove(op: DomOperationRemove): void {
  const { address, items } = op
  const [parent, slot] = resolveParent(address)

  if (isElement(parent)) {
    if (isString(slot)) {
      // If the slot is a string then we're removing a value from an `Option`
      // property of a `struct` so expect the number of items to be 1
      assert(items === 1, `Unexpected items ${items} for slot ${slot}`)
      const node = resolveSlot(parent, slot)
      node.remove()
    } else {
      // If the slot is number then we're removing items from a `Vec` so
      // need to check the `slot` and `items` against the number of children.
      const children = parent.childNodes
      assert(
        slot >= 0 && slot < children.length,
        `Unexpected remove slot ${slot} for element with ${children.length} children`
      )
      assert(
        items > 0 && slot + items <= children.length,
        `Unexpected remove items ${items} for element with ${children.length} children`
      )
      let removed = 0
      while (removed < items) {
        children[slot]?.remove()
        removed += 1
      }
    }
  } else {
    // Removing from a string
    assertNumber(slot)
    const text = parent.textContent ?? ''
    assert(
      slot >= 0 && slot <= text.length,
      `Unexpected remove slot ${slot} for text node of length ${text.length}`
    )
    assert(
      items > 0 && slot + items <= text.length,
      `Unexpected remove items ${slot} for text node of length ${text.length}`
    )
    parent.textContent = text.slice(0, slot) + text.slice(slot + items)
  }
}

/**
 * Apply a replace operation
 */
export function applyReplace(_op: DomOperationReplace): void {
  panic('TODO')
}

/**
 * Apply a move operation
 */
export function applyMove(_op: DomOperationMove): void {
  panic('TODO')
}

/**
 * Apply a transform operation
 *
 * Transform operations allow for a lightweight diff where only the type
 * of the node has changed. See the `diff_transform` function in `rust/src/patches/inlines.rs`
 * This function should be able to apply all the transforms potentially
 * generated on the server.
 */
export function applyTransform(op: DomOperationTransform): void {
  const { address } = op

  const node = resolveNode(address)
  assertElement(node)

  panic('TODO')
}
