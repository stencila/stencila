import { LitElement } from 'lit'

/**
 * A base custom element which provides convenience methods
 */
export default class StencilaElement extends LitElement {
  /**
   * Emit a custom event
   *
   * @param name The name of the custom event
   * @param options Options for the custom event
   * @returns CustomEvent
   */
  emit(name: string, detail = {}, options?: CustomEventInit) {
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
