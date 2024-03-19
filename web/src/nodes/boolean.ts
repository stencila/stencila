import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Entity } from './entity'

import '../ui/nodes/card'

/**
 * Web component representing a Stencila Schema `Boolean` node
 *
 * Note that this extends `Entity`, despite not doing so in Stencila Schema`, to
 * make use of the various `render*View()` methods.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md
 */
@customElement('stencila-boolean')
@withTwind()
export class Boolean extends Entity {
  private bodyStyles = apply([
    'flex justify-center',
    'w-full',
    'py-2 px-6',
    'font-bold font-mono',
  ])

  /**
   * In static view just render the value
   */
  override renderStaticView() {
    return html`<slot></slot>`
  }

  /**
   * In dynamic view, in addition to the value, render a node card.
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-node-card type="Boolean" view="dynamic" ?collapsible=${true}>
        <div slot="body" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-node-card>
    `
  }

  /**
   * In source view, render the same as dynamic view, including the
   * value since that won't be a present in the source usually (given this
   * node type is normally only present in `CodeChunk.outputs` and `CodeExpression.output`).
   */
  override renderSourceView() {
    return html`
      <stencila-ui-node-card type="Boolean" view="source" ?collapsible=${true}>
        <div slot="body" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-node-card>
    `
  }
}
