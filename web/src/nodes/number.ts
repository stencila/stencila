import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/in-flow/block'

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
  private bodyStyles = apply([
    'flex justify-center',
    'w-full',
    'py-2 px-4',
    'font-mono',
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
      <stencila-ui-block-in-flow type="Number" view="dynamic">
        <div slot="body" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-block-in-flow>
    `
  }

  /**
   * In source view, render the same as dynamic view, including the
   * value since that won't be a present in the source usually (given this
   * node type is normally only present in `CodeChunk.outputs` and `CodeExpression.output`).
   */
  override renderSourceView() {
    return html`
      <stencila-ui-block-in-flow type="Number" view="source">
        <div slot="body" class=${this.bodyStyles}><slot></slot></div>
      </stencila-ui-block-in-flow>
    `
  }
}
