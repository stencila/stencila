import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

/**
 * Web component representing an item within a Stencila Schema `Object` node
 */
@customElement('stencila-object-item')
@withTwind()
export class ObjectItem extends LitElement {
  @property()
  key: string

  override render() {
    const itemClasses = apply(['flex flex-row', 'mb-1'])
    const keyClasses = apply([
      'w-full max-w-1/5',
      'p-3',
      'font-mono text-ellipsis',
      'overflow-hidden',
    ])
    return html`
      <div class=${itemClasses}>
        <div class=${keyClasses}>${this.key}:</div>
        <div class="w-full"><slot></slot></div>
      </div>
    `
  }
}
