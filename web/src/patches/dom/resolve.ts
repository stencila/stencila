import { Address, Slot } from '@stencila/stencila'
import HtmlFragment from 'html-fragment'
import { ElementId } from '../../types'
import {
  assert,
  assertElement,
  isElement,
  isName,
  isText,
  panic,
} from '../checks'
import { STRUCT_ATTRIBUTES } from './consts'
import { resolveProxy } from './proxies'

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
    const root = document.body.querySelector('[data-root]')
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
 * Generate a DOM `Element` selector for a name slot
 *
 * Notes:
 *
 * - `:not(meta)` is used to avoid <meta> tags which are only ever added for Microdata
 *   enrichment and which should not be selected for patch application instead of other
 *   elements or attributes
 *
 * - `slot` is used because, even though one of the other attributes may be present
 *   in the original encoding, a Web Component may replace it; specifying `slot` 
 *   here avoids this
 */
export function slotSelector(name: string): string {
  return `[data-prop="${name}"], :not(meta)[itemprop="${name}"], [slot="${name}"]`
}

/**
 * Resolve a slot in a parent DOM node.
 *
 * Note that the `parent` must be an `Element` but that the returned
 * node may be an `Element`, `Attr`, or `Text` DOM node or `null` if
 * the slot could not be resolved.
 */
export function resolveSlot(
  parent: Element,
  slot: Slot
): Element | Attr | Text | undefined {
  if (isName(slot)) {
    // Is the slot represented as a standard attribute e.g. `id`, `value`?
    const attr = parent.attributes.getNamedItem(STRUCT_ATTRIBUTES[slot] ?? slot)
    if (attr !== null) return attr

    // Is the slot represented as a custom attribute on a WebComponent? e.g. `programming-language`
    const customAttr = parent.attributes.getNamedItem(
      slot.replace(/[A-Z]/g, '-$&').toLowerCase()
    )
    if (customAttr !== null) return customAttr

    // Is there a descendant element matching the slot name?
    assertElement(parent)
    const child: Element | null = parent.querySelector(slotSelector(slot))

    // The `text` slot (e.g. on `CodeFragment`) should always represented by the text content of the selected element
    // and is usually "implicit" (so, if there is no explicitly marked text slot, use the parent)
    if (slot === 'text') {
      const elem = child !== null ? child : parent
      if (elem.childNodes.length === 0) {
        // If there is no text node (e.g. the text started off empty) then add one
        const text = document.createTextNode('')
        elem.appendChild(text)
        return text
      } else if (elem.childNodes.length === 1 && isText(elem.childNodes[0])) {
        return elem.childNodes[0]
      } else {
        throw panic(
          `Expected the 'text' slot to resolve to a single text node child`
        )
      }
    }

    // If the child element only has one text node child (e.g. the `label` property of a `Figure`,
    // a `String` represented as `<span>some</span>` in inline content), then return the text DOM node
    // for the operation to be applied to
    if (child?.childNodes.length === 1 && isText(child?.childNodes[0])) {
      return child.childNodes[0]
    }

    // `<meta>` elements are used to represent properties that should not be visible
    // but which are needed, if for nothing other than to provide Microdata for the property.
    // Return the `content` attribute, rather than the element itself.
    if (isElement(child) && child.tagName === 'META') {
      const target = resolveProxy(child)
      if (target) return target

      const content = child.attributes.getNamedItem('content')
      if (content !== null) return content
    }

    if (child !== null) return child

    // Key-value pairs of a <stencila-object> are within a <dl> that has no `itemprop`
    // (because an `Object` is a primitive with no properties). We need to search through
    // these to find a <dt> with content matching the slot name and return the first
    // (and only) child of the <dd>
    if (isObjectElement(parent)) {
      const key = resolveObjectKey(parent, slot)
      if (key !== undefined) {
        const child = key.nextElementSibling
        assertElement(child)
        assert(
          child.childNodes.length === 1,
          'Expected <dd> to have only one child'
        )
        const grandchild = child.childNodes[0]
        assert(
          isElement(grandchild) || isText(grandchild),
          'Expected `Object` value to be element or text'
        )
        return grandchild as Element | Text
      }
    }

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

    return undefined
  } else {
    // Items of a <stencila-array> are within a <ol> that has no `itemprop`
    // (because an `Array` is a primitive with no properties)
    let isArray = false
    if (isArrayElement(parent)) {
      isArray = true
      parent = parent.querySelector('ol') as Element
    }

    // Select the child at the slot index
    const child: ChildNode | undefined = parent.childNodes[slot]
    if (child === undefined) {
      throw panic(
        `Unable to get slot '${slot}' from element with ${parent.childNodes.length} children`
      )
    } else if (isElement(child)) {
      const grandchild = child.childNodes[0]
      // If the parent is an `Array`, the child should be a <li>, so return grandchild
      const isArrayItem =
        isArray &&
        child.tagName === 'LI' &&
        (isElement(grandchild) || isText(grandchild))
      // If the child is a <span> and only has one text node child then return the grandchild text
      const isSimpleSpan =
        (child.tagName === 'SPAN' || child.tagName === 'PRE') &&
        child.childNodes.length === 1 &&
        isText(grandchild)
      if (isArrayItem || isSimpleSpan) {
        return grandchild
      } else {
        return child
      }
    } else if (isText(child)) {
      return child
    } else {
      throw panic(`Unexpected node type '${child.nodeName}' for slot '${slot}'`)
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
): [Element | Attr | Text | undefined, Slot] {
  const targetElement = resolveTarget(target)

  if (address.length === 0) {
    const parentElement = targetElement.parentElement
    if (parentElement === null) {
      throw panic('The target node does not have a parent')
    }
    const slot = Array.from(parentElement.childNodes).indexOf(targetElement)
    return [parentElement, slot]
  }

  let parentNode: Element | Attr | Text | undefined = targetElement
  for (const slot of address.slice(0, -1)) {
    assertElement(parentNode)
    parentNode = resolveSlot(parentNode, slot)
    if (parentNode === undefined) break
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
): Element | Attr | Text | undefined {
  let node: Element | Attr | Text | undefined = resolveTarget(target)

  for (const slot of address) {
    assertElement(node)
    node = resolveSlot(node, slot)
    if (node === undefined) break
  }

  return node
}

/**
 * Resolve the key (represented as a <dt>) for an `Object` (represented as a <dl>)
 */
export function resolveObjectKey(
  object: Element,
  term: string
): Element | undefined {
  const dts = [...object.querySelectorAll('dt')]
  for (const dt of dts) {
    if (dt.textContent === term) {
      return dt
    }
  }
}

/**
 * Is the element a Stencila custom element?
 */
export function isStencilaElement(elem: Element): boolean {
  return elem.tagName.startsWith('STENCILA')
}

/**
 * Does an element represent a Stencila `Object`?
 */
export function isObjectElement(elem: Element): boolean {
  return elem.tagName === 'STENCILA-OBJECT'
}

/**
 * Does an element represent a Stencila `Array`?
 */
export function isArrayElement(elem: Element): boolean {
  return elem.tagName === 'STENCILA-ARRAY'
}

/**
 * Does an element represent the `columns` of a `Datatable`?
 */
export function isDatatableColumns(elem: Element): boolean {
  return (
    elem.parentElement?.tagName === 'STENCILA-DATATABLE' &&
    elem.getAttribute('data-prop') === 'columns'
  )
}

/**
 * Does an element represent the `rows` (as a proxy) of a `Datatable`?
 */
export function isDatatableRows(elem: Element): boolean {
  return (
    elem.parentElement?.tagName === 'STENCILA-DATATABLE' &&
    elem.getAttribute('itemprop') === 'rows'
  )
}

/**
 * Does an element represent the `values` (as a proxy) of a `Datatable`?
 */
export function isDatatableValues(elem: Element): boolean {
  return (
    elem.parentElement?.tagName === 'STENCILA-DATATABLE' &&
    elem.getAttribute('itemprop') === 'values'
  )
}

/**
 * Create a DOM fragment from a HTML string
 *
 * Uses the `html-fragment` package because `document.createRange().createContextualFragment`
 * does not handle elements that must be wrapped e.g. `td`.
 */
export function createFragment(html: string): DocumentFragment {
  return HtmlFragment(html)
}

/**
 * Create a DOM fragment from a HTML string where where each child is to be wrapped in an element
 */
export function createFragmentWrapEach(
  html: string,
  tag: string
): DocumentFragment {
  const fragment = document.createDocumentFragment()
  for (const child of [...createFragment(html).childNodes]) {
    const wrapper = document.createElement(tag)
    wrapper.appendChild(child)
    fragment.appendChild(wrapper)
  }
  return fragment
}
