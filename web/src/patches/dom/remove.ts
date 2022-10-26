import { OperationRemove, Slot, ElementId } from '../../types'
import {
  isName,
  assert,
  assertIndex,
  assertName,
  isAttr,
  isElement,
  assertElement,
} from '../checks'
import { applyRemove as applyRemoveString } from '../string'
import { STRUCT_ATTRIBUTES } from './consts'
import { escapeAttr, unescapeAttr } from './escape'
import { hasProxy } from './proxies'
import {
  isArrayElement,
  isObjectElement,
  resolveObjectKey,
  resolveParent,
  slotSelector,
} from './resolve'

/**
 * Apply a `Remove` operation
 */
export function applyRemove(op: OperationRemove, target?: ElementId): void {
  const { address, items } = op

  const [parent, slot] = resolveParent(address, target)

  if (parent === undefined) {
    console.warn(
      `Unable to resolve address '${address.join(
        ','
      )}'; 'Remove' operation will be ignored'`
    )
  } else if (isElement(parent)) {
    if (isName(slot)) {
      if (isObjectElement(parent)) applyRemoveObject(parent, slot, items)
      else applyRemoveStruct(parent, slot, items)
    } else {
      if (isArrayElement(parent)) applyRemoveArray(parent, slot, items)
      else applyRemoveVec(parent, slot, items)
    }
  } else applyRemoveText(parent, slot, items)
}

/**
 * Apply a `Remove` operation to an element representing a `Object` (key-value pairs).
 */
export function applyRemoveObject(
  struct: Element,
  name: Slot,
  items: number
): void {
  assertName(name)
  assert(items === 1, `Unexpected remove items ${items} for object`)

  const key = resolveObjectKey(struct, name)
  if (key !== undefined) {
    key.nextElementSibling?.remove()
    key.remove()
    return
  }

  console.warn(`Unable to find existing object key "${name}" to remove"`)
}

/**
 * Apply a `Remove` operation to an element representing a `struct`
 */
export function applyRemoveStruct(
  struct: Element,
  name: Slot,
  items: number
): void {
  assertName(name)
  assert(
    items === 1,
    `Unexpected remove items ${items} for option slot '${name}'`
  )

  // Is there a proxy element for the property? If so, apply the operation to its target.
  const target = hasProxy(struct, name)
  if (target) {
    target.applyRemoveStruct(name, items)
    return
  }

  // If the property is represented as a child element then clear it's content
  // and its attributes, other than `data-prop` etc (so that it remains
  // a placeholder if the property is added again later).
  const child = struct.querySelector(slotSelector(name))
  if (child) {
    child.innerHTML = ''
    for (const attr of child.getAttributeNames()) {
      if (attr !== 'data-prop' && attr !== 'itemprop' && attr !== 'slot')
        child.removeAttribute(attr)
    }
    return
  }

  // If the property is represented as an attribute then remove it.
  // Note the fallback to `name` here (even if not in `STRUCT_ATTRIBUTES` we'll remove it).
  const alias = STRUCT_ATTRIBUTES[name] ?? name
  if (struct.hasAttribute(alias)) {
    struct.removeAttribute(alias)
    return
  }

  console.warn(`Unable to find existing struct property "${name}" to remove"`)
}

/**
 * Apply a `Remove` operation to an element representing an `Array`.
 */
export function applyRemoveArray(
  array: Element,
  index: Slot,
  items: number
): void {
  assertIndex(index)

  const ol = array.querySelector('ol')
  assertElement(ol)

  removeChildren(ol, index, items)
}

/**
 * Apply a `Remove` operation to an element representing a `Vec`.
 */
export function applyRemoveVec(vec: Element, index: Slot, items: number): void {
  assertIndex(index)

  removeChildren(vec, index, items)
}

/**
 * Remove children from an element
 */
function removeChildren(elem: Element, index: number, items: number): void {
  const children = elem.childNodes
  assert(
    index >= 0 && index < children.length,
    `Unexpected remove slot '${index}' for ${elem.tagName} element with ${children.length} children`
  )
  assert(
    items > 0 && index + items <= children.length,
    `Unexpected remove items ${items} for ${elem.tagName} element with ${children.length} children`
  )

  let removed = 0
  while (removed < items) {
    children[index]?.remove()
    removed += 1
  }
}

/**
 * Apply a `Remove` operation to a `Text` or `Attr` DOM node representing a `String`
 */
export function applyRemoveText(
  text: Attr | Text,
  index: Slot,
  items: number
): void {
  assertIndex(index)

  const current = text.textContent ?? ''
  const unescaped = isAttr(text) ? unescapeAttr(current) : current
  const updated = applyRemoveString(unescaped, index, items)
  const escaped = isAttr(text) ? escapeAttr(updated) : updated
  text.textContent = escaped
}
