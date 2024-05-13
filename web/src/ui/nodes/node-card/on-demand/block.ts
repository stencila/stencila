import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../../twind'
import '../../../animation/collapsible'
import { ToggleChipMixin } from '../../mixins/toggle-chip'
import { UIBaseCard } from '../base-card'

/**
 * UI block-on-demand
 *
 * A component to render a node-card on demand - i.e. a user requests to see
 * the info rather than just the content of a card.
 */
@customElement('stencila-ui-block-on-demand')
@withTwind()
export class UIBlockOnDemand extends ToggleChipMixin(UIBaseCard) {
  override render() {
    const cardStyles = apply([
      'fit-contents',
      'group',
      'transition duration-400',
      'border border-[transparent]',
      'rounded',
      'font-normal',
      this.view === 'source' ? 'flex flex-col h-full' : '',
      this.toggle && `border-[${this.ui.borderColour}] my-2 mx-auto`,
    ])

    return html`<div class=${`ui-block-on-demand ${cardStyles}`}>
      <div class="relative">
        <stencila-ui-collapsible-animation class=${this.toggle ? 'opened' : ''}>
          ${this.renderHeader()} ${this.renderBody()}
        </stencila-ui-collapsible-animation>
        <div class=${`animated-content`}>${this.renderContent()}</div>
      </div>
    </div>`
  }

  protected override renderBody() {
    const { colour, borderColour } = this.ui
    const bodyStyles = apply([
      'relative',
      'w-full h-full',
      `bg-[${colour}]`,
      `border border-[${borderColour}] rounded-b`,
    ])

    return html`<div class=${bodyStyles}>
      <slot name="body"></slot>
    </div>`
  }

  protected override renderContent() {
    const contentStyles = apply([
      !this.displayContent && this.toggle ? 'hidden' : 'block',
      'relative',
      'transition-[padding] ease-in-out duration-[250ms]',
      'px-0',
      this.toggle && 'p-3',
    ])

    return html`
      <div class=${`${contentStyles}`}>
        ${this.renderChip(this.getIcon(), this.ui)}
        <div class="content-block">
          <slot name="content" class="w-full"></slot>
        </div>
      </div>
    `
  }

  protected override toggleCardDisplay() {
    this.toggle = !this.toggle

    this.shadowRoot.dispatchEvent(
      new CustomEvent(`toggle-${this.nodeId}`, {
        bubbles: true,
        composed: true,
        detail: {
          cardOpen: this.toggle,
          nodeId: this.nodeId,
        },
      })
    )
  }
}
