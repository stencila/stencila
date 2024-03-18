import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

/**
 * Web component representing an item within a Stencila Schema `Array` node
 */
@customElement('stencila-array-item')
export class ArrayItem extends LitElement {
  @property({ type: Number })
  index: number

  override render() {
    return html`${this.index}: <slot></slot>`
  }
}
