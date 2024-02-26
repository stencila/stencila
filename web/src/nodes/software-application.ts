import '@shoelace-style/shoelace/dist/components/icon/icon'

import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `SoftwareApplication` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md
 */
@customElement('stencila-software-application')
@withTwind()
export class SoftwareApplication extends Entity {
  @property()
  name: string

  override render() {
    return html`<div class="my-1 text-xs">
      <span class="items-center flex text-sm pl-6">
        <!-- <sl-icon name="window-fullscreen" class="pr-2"></sl-icon> -->
        ${this.name}
      </span>
    </div>`
  }
}
