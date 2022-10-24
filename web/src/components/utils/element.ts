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
   * Emit a patch event for the closest element having an `id` with
   * a `Replace` operations for one or more properties
   */
  protected changeProperty(...properties: string[]) {
    const ops = properties.map(
      (property) =>
        ({
          type: 'Replace',
          address: [property],
          items: 1,
          length: 1,
          value: this[property],
        } as Operation)
    )
    return this.emitOperations(...ops)
  }

  /**
   * Emit a patch event for the closest element having an `id` with
   * one or more operations
   */
  protected emitOperations(...ops: Operation[]) {
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
