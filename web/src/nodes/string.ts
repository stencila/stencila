import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/node-card/on-demand/in-line'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `String` node
 *
 * Note that this extends `Entity`, despite not doing so in Stencila Schema, to
 * make use of the various `render*View()` methods.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/string.md
 */
@customElement('stencila-string')
@withTwind()
export class String extends Entity {
  /**
   * In dynamic view, render a node card with the value in the content slot.
   */
  override render() {
    const bodyStyles = apply(['w-full'])

    return html`
      <stencila-ui-inline-on-demand type="String" view="dynamic">
        <div slot="content" class=${bodyStyles}>
          <q><slot></slot></q>
        </div>
      </stencila-ui-inline-on-demand>
    `
  }
}
