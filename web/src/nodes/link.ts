import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Link` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/link.md
 */
@customElement('stencila-link')
export class Link extends Entity {
  @property()
  target: string

  override render() {
    return html`<slot></slot>`
  }
}
