import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'
import { ordinalString } from '../../utilities/ordinalString'

import { nodeUi } from './icons-and-colours'

import '../animation/collapsible'

@customElement('stencila-ui-for-block-iteration')
@withTwind()
export class ForBlockIteration extends LitElement {
  /**
   * Whether to show the header of the iteration
   */
  @property({ type: Boolean, attribute: 'show-header' })
  showHeader: boolean = false

  /**
   * The index of the iteration in the `ForBlock`
   */
  @property({ type: Number, attribute: 'iteration-index' })
  iterationIndex: number

  /**
   * Whether this is the last iteration of the `ForBlock`
   */
  @property({ type: Boolean, attribute: 'last-iteration' })
  isLastIteration: boolean = false

  /**
   * Whether the iteration is folded
   */
  @state()
  private isFolded: boolean = false

  override render() {
    const { colour, borderColour, textColour } = nodeUi('ForBlock')

    const headerStyles = apply([
      `${this.showHeader ? 'flex items-center' : 'hidden'}`,
      'px-3 py-2',
      `bg-[${colour}]/40`,
      `text-[${textColour}] text-xs font-sans`,
      'cursor-pointer',
      this.iterationIndex === 0 ? '' : 'border-t',
      this.isFolded ? '' : 'border-b',
      `border-[${borderColour}]/50`,
    ])

    return html`
      <div
        class=${headerStyles}
        @click=${() => (this.isFolded = !this.isFolded)}
      >
        <span>${ordinalString(this.iterationIndex + 1)} iteration</span>
        <stencila-ui-chevron-button
          class="ml-auto"
          default-pos=${this.isFolded ? 'left' : 'down'}
          slot="right-side"
          custom-class="flex items-center"
        ></stencila-ui-chevron-button>
      </div>
      <stencila-ui-collapsible-animation
        class=${this.showHeader ? (!this.isFolded ? 'opened' : '') : 'opened'}
      >
        <div class="p-3 ${this.showHeader ? 'pt-0' : ''}">
          <slot></slot>
        </div>
      </stencila-ui-collapsible-animation>
    `
  }
}
