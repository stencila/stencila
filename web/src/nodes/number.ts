import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/node-card/on-demand/in-line'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Number` node
 *
 * Note that this extends `Entity`, despite not doing so in Stencila Schema, to
 * make use of the various `render*View()` methods.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/number.md
 */
@customElement('stencila-number')
@withTwind()
export class Number extends Entity {
  private bodyStyles = apply(['w-full'])

  /**
   * render a node card with the value in the content slot.
   */
  override render() {
    return html`
      <stencila-ui-inline-on-demand type="Number">
        <div slot="content" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-inline-on-demand>
    `
  }
}
