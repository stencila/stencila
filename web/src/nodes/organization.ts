import '@shoelace-style/shoelace/dist/components/icon/icon'

import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Organization` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md
 */
@customElement('stencila-organization')
@withTwind()
export class Organization extends Entity {
  @property()
  name: string

  override render() {
    return html`<div class="my-1">
      <span class="items-center flex">
        <sl-icon name="building" class="pr-2"></sl-icon>
        ${this.name}
      </span>
    </div>`
  }
}
