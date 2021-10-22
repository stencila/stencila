import { Operation, Patch } from '@stencila/stencila'
import { LitElement, html, TemplateResult } from 'lit'
export { css, html } from 'lit'

export interface StencilaElementConstructor extends CustomElementConstructor {
  hydrate(): void
}

/**
 * A registry of `StencilaElement` available in the browser window.
 *
 * Unfortunately, the standard `window.customElements` object does not provide
 * a way of retrieving a list of elements that have been registered. This object
 * provides for that. This is necessary to be able to call `hydrate()` on each
 * of the registered `StencilaElement` classes after the document has been loaded.
 */
window.stencilaElements = {}

/**
 * Class decorator for registering a Stencila custom element.
 *
 * Similar to Lit's `customElement` but in addition to calling `customElements.define`
 * also adds the class to `stencilaElements`.
 *
 * Intended to be used as a class decorator, but Parcel has issues with the `@` syntax
 * so can be used as a curried function instead e.g. `stencilaElement('stencila-parameter')(Parameter)`
 *
 * @param tagName The name of the custom element's tag. Should be prefixed
 *                with `stencila-`.
 */
export function stencilaElement(
  tagName: string
): (cls: StencilaElementConstructor) => void {
  return (cls: StencilaElementConstructor): void => {
    window.customElements.define(tagName, cls)
    window.stencilaElements[cls.name] = [cls, tagName]
  }
}

export abstract class StencilaElement extends LitElement {
  static hydrate(
    cls: StencilaElementConstructor,
    itemType: string,
    hydrater = (elem: Element, tagName: string) => wrap(elem, tagName)
  ): void {
    console.log(`Hydrating custom element '${cls.name}'`)

    if (!itemType.startsWith('http://')) {
      itemType = `http://stenci.la/${itemType}`
    }

    const tagName = window.stencilaElements[cls.name]?.[1]

    if (tagName !== undefined) {
      document.body
        .querySelectorAll(`[itemtype="${itemType}"]`)
        .forEach((elem) => hydrater(elem, tagName))
    } else {
      console.error(
        `No registered tag name for StencilaElement class '${cls.name}'`
      )
    }
  }

  abstract initialize(): void

  /**
   * Send a patch.
   *
   * Dispatches an event that is listened for, and passed on to the server.
   */
  sendPatch(patch: Patch): void {
    window.dispatchEvent(new CustomEvent('patched', { detail: patch }))
  }

  /**
   * Receive and apply a patch operation.
   *
   * The `resolveReceiver` function in `patches/util.ts` calls this method.
   * If the operation is handled by the custom element it should return `true`.
   * Otherwise, should return `false` in which case, that function will continue to
   * search along the address for a receiver.
   */
  receiveOperation(_op: Operation): boolean {
    return false
  }

  getShadowRoot(): ShadowRoot {
    if (!this.shadowRoot) throw new Error('Shadow root is undefined')
    return this.shadowRoot
  }

  getSlot(which = 0): HTMLElement {
    const elements = this.getShadowRoot()
      .querySelector('slot')
      ?.assignedElements()
    return elements?.[which] as HTMLElement
  }

  getPropertyElem(name: string): HTMLElement {
    const elem = this.getSlot().querySelector(
      `[itemprop="${name}"], [data-itemprop="${name}"]`
    )
    if (!elem) throw new Error('No property')
    return elem as HTMLElement
  }

  getProperty(name: string): string | undefined {
    const elem = this.getPropertyElem(name)
    if (elem.tagName === 'META')
      return elem.getAttribute('content') ?? undefined
    else return elem.textContent ?? undefined
  }

  render(): TemplateResult {
    return html`<slot @slotchange=${() => this.initialize()}></slot>`
  }
}

export function hydrate(): void {
  console.log('Hydrating Stencila custom elements')
  for (const [cls, _tagName] of Object.values(window.stencilaElements)) {
    cls.hydrate()
  }
}

window.addEventListener('load', hydrate)

export function wrap(elem: Element, tagName: string): Element {
  const wrapper = document.createElement(tagName)
  elem.parentElement?.insertBefore(wrapper, elem)
  wrapper.appendChild(elem)
  return wrapper
}
