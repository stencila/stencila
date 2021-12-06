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
import { STRUCT_ATTRIBUTES, STRUCT_ATTRIBUTE_ALIASES } from './consts'
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

  if (parent === undefined) {
    console.warn(
      `Unable to resolve address '${address.join(
        ','
      )}'; 'Add' operation will be ignored'`
    )
  } else if (isElement(parent)) {
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

  // Is the slot represented by an attribute with a different name?
  const alias = STRUCT_ATTRIBUTE_ALIASES[name]
  if (alias !== undefined) {
    struct.setAttribute(alias, escapeAttr(html))
    return
  }

  // Is there a placeholder child element for the property ? If so update it's content.
  const placeholder = struct.querySelector(`[data-itemprop="${name}"]`)
  if (placeholder) {
    placeholder.innerHTML = html
    return
  }

  // Nowhere found on the element to add the property.
  //
  // This may occur for a property that is added to the node but which is not (yet)
  // represented in the HTML (e.g.). For completeness (all patch operations should
  // either successfully apply, or panic) we apply it but invisibly.
  //
  // If the provided HTML does not start with an opening angle bracket `<` then the value
  // being added must be a string (the only value type that does not get wrapped in an element)
  // so add it as a <meta> tag, otherwise wrap it in an invisible <div>.
  console.warn(
    `Unable to find attribute or placeholder element for property "${name}"; will be appended as an invisible element`
  )
  if (!html.startsWith('<')) {
    html = `<meta data-itemprop="${name}" content="${escapeAttr(html)}}">`
  } else {
    html = `<div data-itemprop="${name}" style="display:none">${html}</div>`
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
