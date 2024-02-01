import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

/**
 * A component for displaying information about a `Block` node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-block-infobox')
@withTwind()
export abstract class BlockInfobox extends LitElement {
  @property()
  icon: string = ''

  @property()
  colour: string = ''

  @property()
  override title: string = ''

  override render() {
    const styles = apply([
      'w-full',
      'p-4',
      `border border-[${this.colour}] rounded`,
    ])

    // TODO: design this
    return html`
      <div class=${styles} style="background-color: ${this.colour};">
        <span>
          <sl-icon name=${this.icon} library="stencila"></sl-icon>
          ${this.title}
        </span>
        <slot name="authors"></slot>
      </div>
    `
  }
}
