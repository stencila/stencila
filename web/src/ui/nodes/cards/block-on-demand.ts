import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property, query } from 'lit/decorators'

import { withTwind } from '../../../twind'
import '../../animation/collapsible'
import '../chip'
import { ToggleChipMixin } from '../mixins/toggle-chip'

import { UIBaseCard } from './base-card'

/**
 * UI block-on-demand
 *
 * A component to render a node-card on demand - i.e. a user requests to see
 * the info rather than just the content of a card.
 */
@customElement('stencila-ui-block-on-demand')
@withTwind()
export class UIBlockOnDemand extends ToggleChipMixin(UIBaseCard) {
  @query('slot[name="content"]')
  contentSlot!: HTMLSlotElement

  @property({ type: Boolean })
  removeContentPadding: boolean = false

  protected override toggleChipPosition: string = ''

  private handleContentChange() {
    const elmts = this.contentSlot.assignedElements({ flatten: true })

    if (elmts.length > 0) {
      const contentHeight = elmts[0].getBoundingClientRect().height
      if (!contentHeight) {
        this.noVisibleContent = true
      }
    }
  }

  override render() {
    const cardStyles = apply([
      'fit-contents',
      'group',
      'transition duration-400',
      'border border-[transparent]',
      'rounded',
      'font-normal',
      this.toggle && `border-[${this.ui.borderColour}] my-2 mx-auto`,
    ])
    return html`
      <div class=${`ui-block-on-demand ${cardStyles}`}>
        <div class="relative">
          <stencila-ui-collapsible-animation
            class=${this.toggle ? 'opened' : ''}
          >
            ${this.renderHeader()} ${this.renderAnimatedCardBody()}
          </stencila-ui-collapsible-animation>
          <div class=${`animated-content`}>${this.renderContent()}</div>
        </div>
      </div>
    `
  }

  protected override renderBody() {
    const bodyStyles = apply([
      'relative',
      'w-full h-full',
      `bg-[${this.ui.colour}]`,
      `border-b border-[${this.ui.borderColour}]`,
      `text-[${this.ui.textColour}]`,
    ])

    return html`<div class=${bodyStyles}>
      <slot name="body"></slot>
    </div>`
  }

  protected override renderContent() {
    const contentStyles = apply([
      'transition-[padding] ease-in-out duration-[250ms]',
      this.toggle && (this.removeContentPadding ? '' : 'p-3'),
    ])

    return html`
      <div class=${!this.displayContent && this.toggle ? 'hidden' : 'block'}>
        ${this.renderChip()}
        <div class="content-block ${contentStyles}">
          <slot
            name="content"
            class="relative w-full"
            @slotchange=${this.handleContentChange}
          ></slot>
        </div>
      </div>
    `
  }

  protected toggleCardDisplay() {
    this.toggle = !this.toggle
    this.dispatchToggleEvent()
  }
}
