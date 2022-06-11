import { Slot } from '../../types'
import { assertObject, isElement, JsonValue } from '../checks'
import { applyAddStruct } from './add'
import { STRUCT_ATTRIBUTES } from './consts'
import { escapeAttr, escapeHtml } from './escape'
import { applyRemoveStruct } from './remove'
import { applyReplaceStruct } from './replace'

/**
 * Specification of a proxy element for a property of a `struct`
 */
export interface Proxy {
  // The name of the property being proxied
  propertyName: string

  // Test whether an element matches this proxy
  isProxy: (elem: Element) => boolean

  // Resolve the target element of this proxy
  targetElem: (elem: Element) => Element | null | undefined

  // Is the target of this proxy an attribute on the target element?
  targetIsAttr?: boolean

  // Add a property to the target element
  applyAddStruct?: (
    elem: Element,
    slot: Slot,
    value: JsonValue,
    html: string
  ) => void

  // Remove a property from the target element
  applyRemoveStruct?: (elem: Element, name: Slot, items: number) => void

  // Replace a property on the target element
  applyReplaceStruct?: (
    elem: Element,
    name: Slot,
    items: number,
    value: JsonValue,
    html: string
  ) => void
}

/**
 * A resolved `Proxy` target element
 */
export interface Target {
  elem: Element
  applyAddStruct: (name: Slot, value: JsonValue, html: string) => void
  applyRemoveStruct: (name: Slot, items: number) => void
  applyReplaceStruct: (
    name: Slot,
    items: number,
    value: JsonValue,
    html: string
  ) => void
}

// The `validator` property of a `Parameter` is represented by attributes
// of the sibling <input> element.
const parameterValidator: Proxy = {
  propertyName: 'validator',

  isProxy: (elem: Element) =>
    elem.parentElement?.tagName === 'STENCILA-PARAMETER' &&
    elem.getAttribute('itemprop') === parameterValidator.propertyName,

  targetElem: (elem: Element) =>
    elem.parentElement?.querySelector('input') ??
    elem.parentElement?.querySelector('select'),

  applyAddStruct: (
    elem: Element,
    name: Slot,
    value: JsonValue,
    html: string
  ) => {
    if (name === 'validator') {
      // Adding the entire validator...
      assertObject(value)

      // For `EnumValidator` just add `values` to the <select> target element
      if (value.type === 'EnumValidator') {
        for (const node of value.values as JsonValue[]) {
          const option = document.createElement('option')
          const txt = node == null ? 'null' : node.toString()
          option.setAttribute('value', escapeAttr(txt))
          option.textContent = escapeHtml(txt)
          elem.appendChild(option)
        }
        return
      }

      // For other types add all properties to the <input> target element.
      for (let [name, prop] of Object.entries(value)) {
        switch (name) {
          // Map `type` to <input> type
          case 'type':
            prop =
              {
                BooleanValidator: 'checkbox',
                IntegerValidator: 'number',
                NumberValidator: 'number',
                StringValidator: 'text',
              }[prop as string] ?? 'text'
            break
          // Don't add properties that are not supported
          case 'exclusiveMinimum':
          case 'exclusiveMaximum':
            continue
        }
        applyAddStruct(elem, name, prop, '')
      }
      // `BooleanValidator` also needs `checked` to be updated
      if (value.type === 'BooleanValidator' && value.value === true) {
        elem.setAttribute('checked', '')
      }
    } else {
      // Adding one of the properties of the validator, so apply default
      // function to the target element
      applyAddStruct(elem, name, value, html)
    }
  },

  applyRemoveStruct: (elem: Element, name: Slot, items: number) => {
    if (name === 'validator') {
      // Removing the validator itself

      // For `EnumValidator` remove all <option>s from the <select>
      if (elem.tagName === 'SELECT') {
        elem.innerHTML = ''
        return
      }

      // For other types, remove all attributes potentially on <input>
      for (const attr of [
        'type',
        'checked',
        'min',
        'max',
        'step',
        'minlength',
        'maxlength',
        'pattern',
      ]) {
        elem.removeAttribute(attr)
      }
    } else {
      // Removing one of the properties of the validator, so apply default
      // function to the target element
      applyRemoveStruct(elem, name, items)
    }
  },

  applyReplaceStruct: (
    elem: Element,
    name: Slot,
    items: number,
    value: JsonValue,
    html: string
  ) => {
    parameterValidator.applyRemoveStruct?.(elem, name, items)
    parameterValidator.applyAddStruct?.(elem, name, value, html)
  },
}

// The `default` property of a `Parameter` is represented by the `placeholder` attribute
// of the sibling <input> element.
const parameterDefault: Proxy = {
  propertyName: 'default',

  isProxy: (elem: Element) =>
    elem.parentElement?.tagName === 'STENCILA-PARAMETER' &&
    elem.getAttribute('itemprop') === parameterDefault.propertyName,

  targetElem: (elem: Element) =>
    elem.parentElement?.querySelector('input[name=default]'),

  targetIsAttr: true,

  applyReplaceStruct: (
    elem: Element,
    _name: Slot,
    _items: number,
    value: JsonValue,
    _html: string
  ) => {
    changeValue(elem as HTMLSelectElement | HTMLInputElement, value)
  },
}

// The `value` property of a `Parameter` is represented by the `value` attribute
// of the sibling <input> element.
const parameterValue: Proxy = {
  propertyName: 'value',

  isProxy: (elem: Element) =>
    elem.parentElement?.tagName === 'STENCILA-PARAMETER' &&
    elem.getAttribute('itemprop') === parameterValue.propertyName,

  targetElem: (elem: Element) =>
    elem.parentElement?.querySelector('input[slot=value]') ??
    elem.parentElement?.querySelector('select[slot=value]'),

  targetIsAttr: true,

  applyReplaceStruct: (
    elem: Element,
    _name: Slot,
    _items: number,
    value: JsonValue,
    _html: string
  ) => {
    changeValue(elem as HTMLSelectElement | HTMLInputElement, value)
  },
}

/**
 * Change the value of a <select> or <input> element.
 */
function changeValue(
  elem: HTMLSelectElement | HTMLInputElement,
  value: JsonValue
): void {
  // In addition to changing / removing attributes this sets `value` (or `checked`).
  // This is necessary for any input that has been changed by the user.
  // Without it, those inputs will not show a change.
  if (elem.tagName === 'SELECT') {
    // Set the `selected` attribute on the <option> with
    // matching value
    const val = value == null ? 'null' : value.toString()
    const previouslySelected = elem.querySelector('option[selected]')
    if (previouslySelected !== null) {
      previouslySelected.removeAttribute('selected')
    }
    const newlySelected = elem.querySelector(`option[value="${val}"]`)
    if (newlySelected !== null) {
      newlySelected.setAttribute('selected', '')
    }
    elem.value = val
  } else if (elem.getAttribute('type') === 'checkbox') {
    // Set the `checked` attribute
    if (value === true) {
      elem.setAttribute('checked', '')
    } else {
      elem.removeAttribute('checked')
    }
    ;(elem as HTMLInputElement).checked = value as boolean
  } else {
    const val = value == null ? 'null' : value.toString()
    elem.setAttribute('value', escapeAttr(val))
    elem.value = val
  }
}

export const PROXY_ELEMENTS: Proxy[] = [
  parameterValidator,
  parameterDefault,
  parameterValue,
]

/**
 * Resolve the target DOM node of a proxy
 */
export function resolveProxy(elem: Element): Element | Attr | null | undefined {
  for (const proxy of PROXY_ELEMENTS) {
    if (proxy.isProxy(elem)) {
      const target = proxy.targetElem(elem)
      if (isElement(target)) {
        if (proxy.targetIsAttr) {
          const alias = STRUCT_ATTRIBUTES[proxy.propertyName]
          if (alias !== undefined) {
            return target.getAttributeNode(alias)
          }
        } else {
          return target
        }
      }
    }
  }
}

/**
 * Does an element have a child proxy element for one of its properties
 *
 * Note: At present, for performance reasons, `resolveSlot` and `hasProxy` only
 * check <meta> elements.
 */
export function hasProxy(elem: Element, name: string): Target | undefined {
  for (const child of elem.querySelectorAll('meta')) {
    for (const proxy of PROXY_ELEMENTS) {
      if (proxy.propertyName === name && proxy.isProxy(child)) {
        const target = proxy.targetElem(child)
        if (isElement(target)) {
          return {
            elem: target,
            applyAddStruct: (...args) =>
              (proxy.applyAddStruct ?? applyAddStruct)(target, ...args),
            applyRemoveStruct: (...args) =>
              (proxy.applyRemoveStruct ?? applyRemoveStruct)(target, ...args),
            applyReplaceStruct: (...args) =>
              (proxy.applyReplaceStruct ?? applyReplaceStruct)(target, ...args),
          }
        } else {
          console.warn(
            `Unable to find the target element for the proxy ${name}`
          )
        }
      }
    }
  }
}
