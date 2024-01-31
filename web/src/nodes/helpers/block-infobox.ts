import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

/**
 * A component for displaying information about a `Block` node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-block-infobox')
export abstract class BlockInfobox extends LitElement {
  @property()
  icon: string = ''

  @property()
  override title: string = ''

  override render() {
    // TODO: design this
    return html`<div style="color:red">
      ${this.icon} ${this.title}</span>
      <slot name="authors"></slot>
    </div>`
  }
}
