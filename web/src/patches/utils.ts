import { Address, DomOperation, Slot } from '@stencila/stencila'
import GraphemeSplitter from 'grapheme-splitter'
import { StencilaElement } from '../components/base'
import { ElementId } from '../types'

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

/**
 * Resolve the target of a patch.
 *
 * If a `target` is specified for a patch then return the element
 * with a matching `id`.
 *
 * Otherwise, return the "root" element corresponding to the `root` node of
 * the `Document` in Rust. If unable to find the root node in the
 * `<body>` will log a warning and return the first node child of the body.
 */
export function resolveTarget(target?: ElementId): Element {
  if (target !== undefined) {
    const elem = document.getElementById(target)
    if (elem === null)
      throw panic(
        `Unable to resolve target node; no element with id '${target}'`
      )
    return elem
  } else {
    // It is proposed that `data-root` replace `data-itemscope`. This allows for both
    const root = document.body.querySelector(
      '[data-root], [data-itemscope="root"]'
    )
    if (root === null) {
      console.warn('Unable to resolve root node; using first node of <body>')
      const first = document.body.firstElementChild
      if (first === null)
        throw panic('The <body> does not have a child element!')
      return first
    } else {
      return root
    }
  }
}

/**
 * Resolve a slot in a parent DOM node.
 *
 * Note that the `parent` must be an `Element` but that the returned
 * node may be an `Element`, `Attr`, or `Text` DOM node.
 */
export function resolveSlot(
  parent: Element,
  slot: Slot
): Element | Attr | Text {
  if (isString(slot)) {
    // Select the first descendant element matching the slot name.
    // It is proposed that `data-prop` replace `data-itemprop`.
    // This currently allows for all options.
    assertElement(parent)
    const child: Element | null = parent.querySelector(
      `[data-prop="${slot}"], [data-itemprop="${slot}"], [itemprop="${slot}"]`
    )

    // The `text` slot is always represented by the text content of the selected element
    // and is usually "implicit" (so, if there is no explicitly marked text slot, use the parent)
    if (slot === 'text') {
      const elem = child !== null ? child : parent
      if (elem.childNodes.length === 1 && isText(elem.childNodes[0])) {
        return elem.childNodes[0]
      } else {
        throw panic(
          `Expected the 'text' slot to resolve to a single text node child`
        )
      }
    }

    // `<meta>` elements are used to represent properties that should not be visible
    // but which are needed, if for nothing other than to provide Microdata for the property.
    // Return the `content` attribute, rather than the element itself.
    if (child?.tagName === 'META') {
      const content = child.attributes.getNamedItem('content')
      if (content === null)
        throw panic(
          `Expected <meta> element for slot '${slot}' to have a 'content' attribute`
        )
      return content
    }

    if (child !== null) return child

    // The `content` slot is usually "implicit" (i.e. not represented by an element) but
    // instead represented by the child nodes of the parent element.
    // So, if there is no explicitly marked content slot, return the parent
    if (slot === 'content') return parent

    // See if the slot is represented as a standard HTML attribute e.g. `id`, `value`
    const attr = parent.attributes.getNamedItem(slot)
    if (attr !== null) return attr

    throw panic(`Unable to resolve slot '${slot}'`)
  } else {
    // Select the child at the slot index.
    const child: ChildNode | undefined = parent.childNodes[slot]
    if (child === undefined) {
      throw panic(
        `Unable to get slot '${slot}' from element of with ${parent.childNodes.length} children`
      )
    } else if (isElement(child) || isText(child)) {
      return child
    } else {
      throw panic('Unexpected node type')
    }
  }
}

/**
 * Resolve the parent of the DOM node at the address.
 *
 * Returns the parent DOM node and the target node's slot.
 * This is used for `Add`, `Replace` and `Move` operations where we need
 * the node on which to perform the action and the terminal slot
 * refers to the location within that node to add or replace.
 *
 * If the address is empty, it means that the target node itself is
 * being operated on. In that case, returns the parent element of the target
 * and the index of the target relative to that parent.
 */
export function resolveParent(
  address: Address,
  target?: ElementId
): [Element | Attr | Text, Slot] {
  const targetElement = resolveTarget(target)

  if (address.length === 0) {
    const parentElement = targetElement.parentElement
    if (parentElement === null) {
      throw panic('The target node does not have a parent')
    }
    const slot = Array.from(parentElement.childNodes).indexOf(targetElement)
    return [parentElement, slot]
  }

  let parentNode: Element | Attr | Text = targetElement
  for (const slot of address.slice(0, -1)) {
    assertElement(parentNode)
    parentNode = resolveSlot(parentNode, slot)
  }

  const slot = address[address.length - 1]
  if (slot === undefined) throw panic('Address is too short')

  return [parentNode, slot]
}

/**
 * Resolve the DOM node at the address.
 */
export function resolveNode(
  address: Address,
  target?: ElementId
): Element | Attr | Text {
  let node: Element | Attr | Text = resolveTarget(target)

  for (const slot of address) {
    assertElement(node)
    node = resolveSlot(node, slot)
  }

  return node
}

/**
 * Resolve the DOM Element that should receive an operation.
 *
 * Searches along the address for a `<stencila-*>` element
 * that will receive the operation. If such an element is
 * found returns `true` (in which case any further handling of the
 * operation should probably be avoided).
 */
export function resolveReceiver(
  address: Address,
  op: DomOperation,
  target?: ElementId
): boolean {
  let node: Element | Attr | Text = resolveTarget(target)

  let index = 0
  while (isElement(node)) {
    const parent = node.parentElement
    if (parent?.tagName.toLowerCase().startsWith('stencila-')) {
      const elem = parent as StencilaElement
      if (elem.receiveOperation(op)) return true
    }

    const slot = address[index]
    if (slot === undefined) {
      break
    }
    node = resolveSlot(node, slot)

    index++
  }

  return false
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
