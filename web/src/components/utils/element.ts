import { LitElement } from 'lit'
import { Operation, Patch } from '../../types'

/**
 * A base custom element which provides convenience methods
 */
export default class StencilaElement extends LitElement {
  /**
   * Get the closest ancestor element matching the selector
   *
   * Based on https://stackoverflow.com/q/54520554
   */
  protected closestElement(selector: string, el = this) {
    return (
      // @ts-ignore
      (el && el != document && el != window && el.closest(selector)) ||
      // @ts-ignore
      this.closestElement(selector, el.getRootNode().host)
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
    let target = this.closestElement('[id]')?.id
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
