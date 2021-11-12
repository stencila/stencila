import { OperationAdd, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
  assertIndex,
  assertName,
  assertString,
  isAttr,
  isElement,
  isName,
  JsonValue,
  panic,
} from '../checks'
import { applyAdd as applyAddString } from '../string'
import { STRUCT_ATTRIBUTES } from './consts'
import { escapeAttr, unescapeAttr, unescapeHtml } from './escape'
import { createFragment, resolveParent } from './resolve'

/**
 * Apply an add operation
 */
export function applyAdd(op: OperationAdd, target?: ElementId): void {
  const { address } = op
  const value = op.value as JsonValue
  const html = op.html ?? value

  const [parent, slot] = resolveParent(address, target)

  if (isElement(parent)) {
    assertString(html)
    if (isName(slot)) applyAddStruct(parent, slot, html)
    else applyAddVec(parent, slot, html)
  } else {
    assertString(value)
    applyAddText(parent, slot, value)
  }
}

/**
 * Apply an `Add` operation to an element representing a `struct` (with an optional property).
 */
export function applyAddStruct(
  struct: Element,
  name: Slot,
  html: string
): void {
  assertName(name)

  // Is the property designated to be represented as an attribute?
  if (STRUCT_ATTRIBUTES.includes(name)) {
    struct.setAttribute(name, escapeAttr(html))
    return
  }

  // Is there a placeholder child element for the property ? If so update it's content.
  const placeholder = struct.querySelector(`[data-itemprop="${name}"]`)
  if (placeholder) {
    placeholder.innerHTML = html
    return
  }

  // Otherwise, emit a warning but still append as a child.
  // If the provided HTML does not start with an opening angle bracket `<` then the value
  // being added must be a string (the only value type that does not get wrapped in an element)
  // so wrap it.
  console.warn(
    `Unable to find attribute or placeholder element for property "${name}"; will be appended`
  )
  if (!html.startsWith('<')) {
    html = `<span data-itemprop="${name}">${html}</span>`
  }
  const fragment = createFragment(html)
  struct.appendChild(fragment)
}

/**
 * Apply an `Add` operation to an element representing a `Vec`.
 */
export function applyAddVec(vec: Element, index: Slot, html: string): void {
  assertIndex(index)

  const fragment = createFragment(html)
  const children = vec.childNodes
  if (index === children.length) {
    vec.appendChild(fragment)
  } else {
    const sibling = vec.childNodes[index]
    if (sibling === undefined)
      throw panic(
        `Unexpected add slot '${index}' for element with ${children.length} children`
      )
    vec.insertBefore(fragment, sibling)
  }
}

/**
 * Apply an `Add` operation to a `Text` or `Attr` DOM node representing a `String`
 */
export function applyAddText(
  text: Text | Attr,
  index: Slot,
  value: string
): void {
  assertIndex(index)

  const current = text.textContent ?? ''
  const unescaped = isAttr(text) ? unescapeAttr(current) : unescapeHtml(current)
  const updated = applyAddString(unescaped, index, value)
  // It seems that, because setting textContent (?), it is not necessary to escape innerHTML
  const escaped = isAttr(text) ? escapeAttr(updated) : updated
  text.textContent = escaped
}
