import { Address, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import { assertElement, isElement, isName, isText, panic } from '../checks'

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
  if (isName(slot)) {
    // Is the slot represented as a standard attribute e.g. `id`, `value`?
    const attr = parent.attributes.getNamedItem(slot)
    if (attr !== null) return attr

    // Is the slot represented as a custom attribute on a WebComponent? e.g. `programming-language`
    const dataAttr = parent.attributes.getNamedItem(
      slot.replace(/[A-Z]/g, '-$&').toLowerCase()
    )
    if (dataAttr !== null) return dataAttr

    // Is there a descendant element matching the slot name?
    // It is proposed that `data-prop` replace `data-itemprop`. This currently allows for all options.
    assertElement(parent)
    const child: Element | null = parent.querySelector(
      `[data-prop="${slot}"], [data-itemprop="${slot}"], [itemprop="${slot}"]`
    )

    // The `text` slot should always represented by the text content of the selected element
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

    // If the element only has one text node child then return it as the slot
    // TODO: Consider amalgamating this with above.
    if (
      (child?.tagName === 'SPAN' || child?.tagName === 'PRE') &&
      child?.childNodes.length === 1 &&
      isText(child?.childNodes[0])
    ) {
      return child.childNodes[0]
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

    // The `content`, `items`, `rows` and `cell` slots are usually "implicit"
    // (i.e. not represented by an element) but instead represented by the child nodes of
    // the parent element. So, if there is no explicitly marked content slot, return the parent
    // (which will probably then be indexed by a number slot to get the child)
    if (
      slot === 'content' ||
      (slot === 'items' &&
        (parent.tagName === 'UL' || parent.tagName === 'OL')) ||
      (slot === 'rows' && parent.tagName === 'TABLE') ||
      (slot === 'cells' && parent.tagName === 'TR')
    )
      return parent

    throw panic(`Unable to resolve slot '${slot}'`)
  } else {
    // Select the child at the slot index.
    const child: ChildNode | undefined = parent.childNodes[slot]
    if (child === undefined) {
      throw panic(
        `Unable to get slot '${slot}' from element with ${parent.childNodes.length} children`
      )
    } else if (isElement(child)) {
      // If the element is a `<span>` and only has one text node child then return it as the slot
      if (
        (child.tagName === 'SPAN' || child.tagName === 'PRE') &&
        child.childNodes.length === 1 &&
        isText(child.childNodes[0])
      ) {
        return child.childNodes[0]
      } else {
        return child
      }
    } else if (isText(child)) {
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
 * Create a DOM fragment from a HTML string
 */
export function createFragment(html: string): DocumentFragment {
  return document.createRange().createContextualFragment(html)
}
