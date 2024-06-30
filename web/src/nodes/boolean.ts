import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/node-card/on-demand/block'

/**
 * Web component representing a Stencila Schema `Boolean` node
 *
 * Note that this extends `Entity`, despite not doing so in Stencila Schema, to
 * make use of the various `render*View()` methods.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md
 */
@customElement('stencila-boolean')
@withTwind()
export class Boolean extends Entity {
  private bodyStyles = apply(['w-full'])

  /**
   * In dynamic view, render a node card with the value in the content slot.
   */
  override render() {
    return html`
      <stencila-ui-block-on-demand type="Boolean" view="dynamic">
        <div slot="content" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-block-on-demand>
    `
  }
}
