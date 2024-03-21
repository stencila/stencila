import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

/**
 * Web component representing an item within a Stencila Schema `Array` node
 */
@customElement('stencila-array-item')
@withTwind()
export class ArrayItem extends LitElement {
  @property({ type: Number })
  index: number

  override render() {
    const indexClasses = apply([
      'w-full max-w-8',
      'py-2',
      'font-mono text-ellipsis',
      'overflow-hidden',
    ])

    return html`
      <div class="flex flex-row">
        <div class=${indexClasses}>${this.index}:</div>
        <div class="w-full"><slot></slot></div>
      </div>
    `
  }
}
