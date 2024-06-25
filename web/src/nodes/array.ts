import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/card'

import { withTwind } from '../twind'

import { Entity } from './entity'

import './array-item'
import '../ui/nodes/node-card/on-demand/block'

/**
 * Web component representing a Stencila Schema `Array` node
 *
 * Note that this extends `Entity`, despite not doing so in Stencila Schema, to
 * make use of the various `render*View()` methods.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array.md
 */
@customElement('stencila-array')
@withTwind()
export class Array extends Entity {
  override render() {
    return html`
      <stencila-ui-block-on-demand type="Array" view="dynamic">
        <div slot="content">
          <slot></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
