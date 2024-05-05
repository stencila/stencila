import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/node-card/on-demand/block'

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
  private bodyStyles = apply(['w-full', 'p-2', 'font-mono'])

  /**
   * In static view just render the value
   */
  override renderStaticView() {
    return html`<slot></slot>`
  }

  /**
   * In dynamic view, render a node card with the value in the content slot.
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-block-on-demand type="Number" view="dynamic">
        <div slot="content" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In source view, render the same as dynamic view, including the
   * value since that won't be a present in the source usually (given this
   * node type is normally only present in `CodeChunk.outputs` and `CodeExpression.output`).
   */
  override renderSourceView() {
    return html`
      <stencila-ui-block-on-demand type="Number" view="source">
        <div slot="body" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-block-on-demand>
    `
  }
}
