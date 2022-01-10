import { OperationAdd, Slot } from '@stencila/stencila'
import { ElementId } from '../../types'
import {
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
import { applyAdd as applyAddString } from '../string'
import { STRUCT_ATTRIBUTES } from './consts'
import { escapeAttr, unescapeAttr, unescapeHtml } from './escape'
import { hasProxy } from './proxies'
import {
  createFragment,
  createFragmentWrapEach,
  isArrayElement,
  isObjectElement,
  resolveParent,
  slotSelector,
} from './resolve'

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
    if (isName(slot)) {
      if (isObjectElement(parent)) applyAddObject(parent, slot, html)
      else applyAddStruct(parent, slot, value, html)
    } else {
      if (isArrayElement(parent)) applyAddArray(parent, slot, html)
      else applyAddVec(parent, slot, html)
    }
  } else {
    assertString(value)
    applyAddText(parent, slot, value)
  }
}

/**
 * Apply an `Add` operation to an element representing an `Object` (key-value pairs)
 */
export function applyAddObject(
  object: Element,
  name: Slot,
  html: string
): void {
  assertName(name)

  const dl = object.querySelector('dl')
  assertElement(dl)

  const fragment = createFragment(`<dt>${name}</dt><dd>${html}</dd>`)
  dl.appendChild(fragment)
}

/**
 * Apply an `Add` operation to an element representing a `struct` (with an optional property).
 */
export function applyAddStruct(
  struct: Element,
  name: Slot,
  value: JsonValue,
  html: string
): void {
  assertName(name)

  // Is there a proxy element for the property? If so, apply the operation to its target.
  const target = hasProxy(struct, name)
  if (target) {
    target.applyAddStruct(name, value, html)
    return
  }

  // Is there a placeholder element for the property? If so update it's content.
  // Takes precedence over adding as an attribute.
  const placeholder = struct.querySelector(slotSelector(name))
  if (placeholder) {
    placeholder.innerHTML = html
    return
  }

  // Is the slot represented by an attribute?
  const alias = STRUCT_ATTRIBUTES[name]
  if (alias !== undefined) {
    let attr = ''
    if (value == null) attr = 'null'
    else if (typeof value === 'object' && !Array.isArray(value)) {
      if (value.type === 'Date') {
        // Use the ISO date string as the attribute
        attr = value.value as string
      }
    } else attr = value.toString()

    struct.setAttribute(alias, escapeAttr(attr))
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
 * Apply an `Add` operation to an element representing an `Array`.
 */
export function applyAddArray(array: Element, index: Slot, html: string): void {
  assertIndex(index)

  const ol = array.querySelector('ol')
  assertElement(ol)

  const fragment = createFragmentWrapEach(html, 'li')
  addChildren(ol, index, fragment)
}

/**
 * Apply an `Add` operation to an element representing a `Vec`.
 */
export function applyAddVec(vec: Element, index: Slot, html: string): void {
  assertIndex(index)

  const fragment = createFragment(html)
  addChildren(vec, index, fragment)
}

/**
 * Add children to an element
 */
function addChildren(
  elem: Element,
  index: number,
  fragment: DocumentFragment
): void {
  const children = elem.childNodes
  if (index === children.length) {
    elem.appendChild(fragment)
  } else {
    const sibling = elem.childNodes[index]
    if (sibling === undefined)
      throw panic(
        `Unexpected add slot '${index}' for ${elem.tagName} element with ${children.length} children`
      )
    elem.insertBefore(fragment, sibling)
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
