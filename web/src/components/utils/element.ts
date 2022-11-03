import { LitElement } from 'lit'
import { Operation, Patch } from '../../types'

/**
 * A base custom element which provides convenience methods
 */
export default class StencilaElement extends LitElement {
  /**
   * Get an element by id from within the shadow DOM of this element
   *
   * This is mainly provided for so that patches that target a node
   * with a specific id can find those id's within the shadow DOM
   * of nested custom elements (these are not accessible by
   * `document.getElementById()`)
   */
  getElementById(elementId: string): HTMLElement | null {
    let elem = this.shadowRoot?.getElementById(elementId) ?? null
    if (elem === null) {
      for (const child of [...(this.shadowRoot?.querySelectorAll('*') ?? [])]) {
        elem =
          child.getElementById?.(elementId) ??
          child.querySelector('#' + elementId) ??
          null
        if (elem !== null) {
          return elem
        }
      }
    }
    return elem
  }

  /**
   * Get the closest ancestor element matching the selector
   *
   * Compared to the native `closest()` method, this will traverse out of
   * Shadow DOMs.
   *
   * Based on https://stackoverflow.com/q/54520554
   */
  static closestElement(el: Element, selector: string) {
    return (
      // @ts-ignore
      (el && el != document && el != window && el.closest(selector)) ||
      // @ts-ignore
      StencilaElement.closestElement(el.getRootNode().host, selector)
    )
  }

  /**
   * Change a property and emit an operation representing the change
   */
  protected changeProperty(property: string, value: unknown) {
    return this.changeProperties([[property, value]])
  }

  /**
   * Change several properties and emit an operation representing the changes
   */
  protected changeProperties(properties: [string, unknown][]) {
    const ops = properties.map(([property, value]) => {
      if (value === null || Number.isNaN(value)) {
        value = undefined
      }

      this[property] = value

      const op: Operation =
        value === undefined
          ? {
              type: 'Remove',
              address: [property],
              items: 1,
            }
          : {
              type: 'Replace',
              address: [property],
              items: 1,
              length: 1,
              value,
            }

      return op
    })

    return this.emitOps(...ops)
  }

  /**
   * Emit a patch event for the closest element having an `id` with
   * one or more operations
   */
  protected emitOps(...ops: Operation[]) {
    const target = StencilaElement.closestElement(this, '[id]')?.id
    return this.emitPatch({
      target,
      ops,
    })
  }

  /**
   * Emit a patch
   */
  protected async emitPatch(patch: Patch) {
    return this.emit('stencila-document-patch', patch)
  }

  /**
   * Emit a custom event
   *
   * @param name The name of the custom event
   * @param detail The event details
   * @param options Options for the custom event
   * @returns CustomEvent
   */
  protected emit(name: string, detail = {}, options?: CustomEventInit) {
    const event = new CustomEvent(name, {
      bubbles: true,
      cancelable: false,
      composed: true,
      detail,
      ...options,
    })
    this.dispatchEvent(event)
    return event
  }
}
