import HtmlFragment from 'html-fragment'
import { OperationReplace, Slot, ElementId } from '../../types'
import {
  assert,
  assertElement,
  assertIndex,
  assertName,
  assertString,
  isAttr,
  isElement,
  isName,
  JsonValue,
  panic,
} from '../checks'
import { applyReplace as applyReplaceString } from '../string'
import { applyAddObject, applyAddStruct } from './add'
import { escapeAttr, unescapeAttr } from './escape'
import {
  createFragment,
  createFragmentWrapEach,
  isArrayElement,
  isObjectElement,
  resolveObjectKey,
  resolveParent,
  slotSelector,
} from './resolve'

/**
 * Apply a replace operation
 */
export function applyReplace(op: OperationReplace, target?: ElementId): void {
  const { address, items, length } = op
  const value = op.value as JsonValue
  const html = op.html ?? value

  const [parent, slot] = resolveParent(address, target)

  if (parent === undefined) {
    console.warn(
      `Unable to resolve address '${address.join(
        ','
      )}'; 'Replace' operation will be ignored'`
    )
  } else if (isElement(parent)) {
    assertString(html)
    if (isName(slot)) {
      if (isObjectElement(parent)) applyReplaceObject(parent, slot, items, html)
      else applyReplaceStruct(parent, slot, items, value, html)
    } else {
      if (isArrayElement(parent))
        applyReplaceArray(parent, slot, items, length, html)
      else applyReplaceVec(parent, slot, items, length, html)
    }
  } else {
    let text: string
    if (typeof value === 'string') {
      text = value
    } else if (value === null) {
      throw panic('Got a null value')
    } else {
      text = value.toString()
    }
    applyReplaceText(parent, slot, items, text)
  }
}

/**
 * Apply a `Replace` operation to an element representing a `Object` (key-value pairs)
 */
export function applyReplaceObject(
  object: Element,
  name: Slot,
  items: number,
  html: string
): void {
  assertName(name)
  assert(items === 1, `Unexpected replace items ${items} for object`)

  const key = resolveObjectKey(object, name)
  if (key !== undefined) {
    const fragment = createFragment(`<dd>${html}</dd>`)
    key.nextElementSibling?.replaceWith(fragment)
  } else {
    console.warn('Unable to find existing object key to replace; will add')
    applyAddObject(object, name, html)
  }
}

/**
 * Apply a `Replace` operation to an element representing a `struct`
 */
export function applyReplaceStruct(
  struct: Element,
  name: Slot,
  items: number,
  value: JsonValue,
  html: string
): void {
  assertName(name)
  assert(
    items === 1,
    `Unexpected replace items ${items} for option slot '${name}'`
  )

  // Is there an element for the property? If so, replace it with the new HTML
  // but retain any `slot` or `data-prop` attributes.
  const existing = struct.querySelector(slotSelector(name))
  if (existing) {
    const replacement = HtmlFragment(html).firstElementChild
    if (replacement) {
      const slot = existing.getAttribute('slot')
      if (slot) replacement.setAttribute('slot', slot)
      const prop = existing.getAttribute('data-prop')
      if (prop) replacement.setAttribute('data-prop', prop)

      existing.replaceWith(replacement)
    }
    return
  }

  // Otherwise, delegate to `applyAddStruct` which has the same logic as needed here for attributes
  applyAddStruct(struct, name, value, html)
}

/**
 * Apply a `Replace` operation to an element representing an `Array`
 */
export function applyReplaceArray(
  array: Element,
  index: Slot,
  items: number,
  length: number,
  html: string
): void {
  assertIndex(index)

  const ol = array.querySelector('ol')
  assertElement(ol)

  const fragment = createFragmentWrapEach(html, 'li')
  replaceChildren(ol, index, items, length, fragment)
}

/**
 * Apply a `Replace` operation to an element representing a `Vec`
 */
export function applyReplaceVec(
  vec: Element,
  index: Slot,
  items: number,
  length: number,
  html: string
): void {
  assertIndex(index)

  const fragment = createFragment(html)
  replaceChildren(vec, index, items, length, fragment)
}

/**
 * Replace children in an element
 */
function replaceChildren(
  elem: Element,
  index: number,
  items: number,
  length: number,
  fragment: DocumentFragment
): void {
  const children = elem.childNodes
  if (children.length === 0) {
    elem.appendChild(fragment)
  } else {
    const sibling = children[index]
    if (sibling === undefined) {
      throw panic(
        `Unexpected replace slot '${index}' for ${elem.tagName} element with ${children.length} children`
      )
    }
    elem.insertBefore(fragment, sibling)

    let removed = 0
    while (removed < items) {
      children[index + length]?.remove()
      removed += 1
    }
  }
}

/**
 * Apply a `Replace` operation to a `Text` or `Attr` DOM node representing a `String`
 */
export function applyReplaceText(
  text: Attr | Text,
  index: Slot,
  items: number,
  value: string
): void {
  assertIndex(index)

  const current = text.textContent ?? ''
  const unescaped = isAttr(text) ? unescapeAttr(current) : current
  const updated = applyReplaceString(unescaped, index, items, value)
  const escaped = isAttr(text) ? escapeAttr(updated) : updated
  text.textContent = escaped
}
