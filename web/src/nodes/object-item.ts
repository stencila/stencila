import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
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
    const keyClasses = apply([
      'flex items-center',
      'w-full max-w-1/5',
      'font-mono text-ellipsis',
      'overflow-hidden',
    ])

    // Note: use tooltip because key may have a maximum width
    return html`
      <div class="flex flex-row">
        <sl-tooltip content="${this.key}">
          <div class=${keyClasses}>${this.key}:</div>
        </sl-tooltip>
        <div class="w-full"><slot></slot></div>
      </div>
    `
  }
}
