import { OperationReplace, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
  assert,
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
import { applyAddStruct } from './add'
import { escapeAttr, unescapeAttr, unescapeHtml } from './escape'
import { createFragment, resolveParent } from './resolve'

/**
 * Apply a replace operation
 */
export function applyReplace(op: OperationReplace, target?: ElementId): void {
  const { address, items } = op
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
    if (isName(slot)) applyReplaceStruct(parent, slot, items, html)
    else applyReplaceVec(parent, slot, items, html)
  } else {
    assertString(value)
    applyReplaceText(parent, slot, items, value)
  }
}

/**
 * Apply a `Replace` operation to an element representing a `struct`
 */
export function applyReplaceStruct(
  struct: Element,
  name: Slot,
  items: number,
  html: string
): void {
  assertName(name)
  assert(
    items === 1,
    `Unexpected replace items ${items} for option slot '${name}'`
  )

  // Simply delegate to `applyAddStruct` which has the same logic as needed here
  applyAddStruct(struct, name, html)
}

/**
 * Apply a `Replace` operation to an element representing a `Vec`
 */
export function applyReplaceVec(
  vec: Element,
  index: Slot,
  items: number,
  html: string
): void {
  assertIndex(index)

  const fragment = createFragment(html)
  const children = vec.childNodes
  if (children.length === 0) {
    vec.appendChild(fragment)
  } else {
    const child = children[index]
    if (child === undefined) {
      throw panic(
        `Unexpected replace slot '${index}' for element with ${children.length} children`
      )
    }
    vec.insertBefore(fragment, child)

    let removed = 0
    while (removed < items) {
      children[index + 1]?.remove()
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
  const unescaped = isAttr(text) ? unescapeAttr(current) : unescapeHtml(current)
  const updated = applyReplaceString(unescaped, index, items, value)
  // It seems that, because setting textContent (?), it is not necessary to escape innerHTML
  const escaped = isAttr(text) ? escapeAttr(updated) : updated
  text.textContent = escaped
}
