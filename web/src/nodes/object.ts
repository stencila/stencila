import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/node-card/on-demand/in-line'

import { withTwind } from '../twind'

import { Entity } from './entity'
import './object-item'

/**
 * Web component representing a Stencila Schema `Object` node
 *
 * Note that this extends `Entity`, despite not doing so in Stencila Schema, to
 * make use of the various `render*View()` methods.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/object.md
 */
@customElement('stencila-object')
@withTwind()
export class Object extends Entity {
  /**
   * render a node card with the value in the content slot.
   */
  override render() {
    return html`
      <stencila-ui-inline-on-demand type="Object">
        <div slot="content">
          <slot></slot>
        </div>
      </stencila-ui-inline-on-demand>
    `
  }
}
