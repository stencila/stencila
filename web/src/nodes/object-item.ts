import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

/**
 * Web component representing an item within a Stencila Schema `Object` node
 */
@customElement('stencila-object-item')
export class ObjectItem extends LitElement {
  @property()
  key: string

  override render() {
    return html`${this.key}: <slot></slot>`
  }
}
