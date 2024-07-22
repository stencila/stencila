import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { getOrdinalString } from '../../../utility/ordinal'
import { nodeUi } from '../icons-and-colours'

import '../../animation/collapsible'

@customElement('stencila-ui-iteration-section')
@withTwind()
export class ForBlockIteration extends LitElement {
  /**
   * Whether or not the header element should be visible,
   * Based on whether the `ForBlock` node card is open.
   */
  @property({ type: Boolean, attribute: 'show-header' })
  showHeader: boolean = false

  @property({ type: Number, attribute: 'iteration-index' })
  iterationIndex: number

  @property({ type: Boolean, attribute: 'last-iteration' })
  isLastItertation: boolean = false

  @state()
  private isFolded: boolean = false

  override render() {
    const { colour, textColour } = nodeUi('ForBlock')

    const headerStyles = apply([
      `${this.showHeader ? 'flex items-center' : 'hidden'}`,
      'px-3 py-2',
      `bg-[${colour}]`,
      `text-[${textColour}] text-sm font-sans`,
      'cursor-pointer',
      // this.isLastItertation && this.isFolded ? '' : 'border-b border-black/20',
    ])

    return html`
      <div
        class=${headerStyles}
        @click=${() => (this.isFolded = !this.isFolded)}
      >
        <span>${getOrdinalString(this.iterationIndex + 1)} Iteration</span>
        <stencila-chevron-button
          class="ml-auto"
          default-pos=${this.isFolded ? 'left' : 'down'}
          slot="right-side"
          custom-class="flex items-center"
        ></stencila-chevron-button>
      </div>
      <stencila-ui-collapsible-animation
        class=${this.showHeader ? (!this.isFolded ? 'opened' : '') : 'opened'}
      >
        <div class="${this.showHeader ? 'p-3' : ''}">
          <slot></slot>
        </div>
      </stencila-ui-collapsible-animation>
    `
  }
}
