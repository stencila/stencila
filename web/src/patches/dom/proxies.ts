import { Slot } from '@stencila/stencila'
import { assertObject, isElement, JsonValue } from '../checks'
import { applyAddStruct } from './add'
import { STRUCT_ATTRIBUTES } from './consts'
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
  resolveTarget: (elem: Element) => Element | null | undefined

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

  resolveTarget: (elem: Element) => elem.parentElement?.querySelector('input'),

  applyAddStruct: (
    elem: Element,
    name: Slot,
    value: JsonValue,
    html: string
  ) => {
    if (name === 'validator') {
      // Adding an entire validator so add all its properties to
      // the target element.
      assertObject(value)
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
      // Removing the validator itself, so remove all attributes potentially
      // added by it
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

// The `value` property of a `Parameter` is represented by the `value` attribute
// of the sibling <input> element.
const parameterValue: Proxy = {
  propertyName: 'value',

  isProxy: (elem: Element) =>
    elem.parentElement?.tagName === 'STENCILA-PARAMETER' &&
    elem.getAttribute('itemprop') === parameterValue.propertyName,

  resolveTarget: (elem: Element) => elem.parentElement?.querySelector('input'),
  targetIsAttr: true,

  applyReplaceStruct: (
    elem: Element,
    _name: Slot,
    _items: number,
    value: JsonValue,
    _html: string
  ) => {
    // Need to set the `checked` attribute for `checkbox` type elements
    if (elem.getAttribute('type') === 'checkbox') {
      if (value === true) {
        elem.setAttribute('checked', '')
      } else {
        elem.removeAttribute('checked')
      }
    } else {
      const attr = value == null ? 'null' : value.toString()
      elem.setAttribute('value', attr)
    }
  },
}

export const PROXY_ELEMENTS: Proxy[] = [parameterValidator, parameterValue]

/**
 * Resolve the target DOM node of a proxy
 */
export function resolveProxy(
  elem: Element,
  name?: string
): Element | Attr | null | undefined {
  for (const proxy of PROXY_ELEMENTS) {
    if (proxy.isProxy(elem)) {
      const target = proxy.resolveTarget(elem)
      if (isElement(target)) {
        if (proxy.targetIsAttr) {
          const alias = STRUCT_ATTRIBUTES[proxy.propertyName]
          if (alias) {
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
        const target = proxy.resolveTarget(child)
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
        }
      }
    }
  }
}
