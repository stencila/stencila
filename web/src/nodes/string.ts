import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/cards/inline-on-demand'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `String` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md
 */
@customElement('stencila-string')
@withTwind()
export class String extends Entity {
  override render() {
    return html`
      <stencila-ui-inline-on-demand type="String">
        <div slot="content" class="w-full">
          <q><slot></slot></q>
        </div>
      </stencila-ui-inline-on-demand>
    `
  }
}
