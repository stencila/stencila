import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { ToggleMarkerMixin } from '../mixins/toggle-marker'

import { UIBaseCard } from './base-card'

import '../../animation/collapsible'

/**
 * UI block-on-demand
 *
 * A component to render a node-card on demand - i.e. a user requests to see
 * the info rather than just the content of a card.
 */
@customElement('stencila-ui-block-on-demand')
@withTwind()
export class UIBlockOnDemand extends ToggleMarkerMixin(UIBaseCard) {
  @property({ type: Boolean })
  removeContentPadding: boolean = false

  protected override toggleMarkerPosition: string = ''

  override render() {
    const hasBorder = this.depth > 0 && this.toggle

    const cardStyles = apply([
      'group',
      'transition duration-400',
      hasBorder
        ? `rounded border border-[${this.ui.borderColour}] my-2 mx-auto`
        : '',
    ])

    return html`
      <div class=${cardStyles}>
        <div class="relative">
          <stencila-ui-collapsible-animation
            class=${this.toggle ? 'opened' : ''}
          >
            ${this.renderHeader()} ${this.renderAnimatedCardBody()}
          </stencila-ui-collapsible-animation>
          ${this.renderContent()}
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
        ${this.renderMarker()}
        <div class="content-block ${contentStyles}">
          <slot name="content" class="relative w-full"></slot>
        </div>
      </div>
    `
  }

  protected toggleCardDisplay() {
    this.toggle = !this.toggle
    this.dispatchToggleEvent()
  }
}
