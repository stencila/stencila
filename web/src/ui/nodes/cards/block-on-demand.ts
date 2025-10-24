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
  @property({ attribute: 'no-content-padding', type: Boolean })
  noContentPadding: boolean = false

  protected override toggleMarkerPosition: string = ''

  override render() {
    const hasBorder = (this.depth > 0 || !this.hasRoot) && this.toggle

    const cardStyles = apply([
      'group',
      'transition duration-400',
      'h-full',
      hasBorder
        ? `rounded border border-[${this.ui.borderColour}] my-2 mx-auto`
        : '',
    ])

    return html`
      <div class=${cardStyles}>
        <div class="relative h-full">
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
    const hasBorder = (this.depth > 0 || !this.hasRoot) && this.toggle

    const bodyStyles = apply(
      'relative',
      'w-full h-full',
      `bg-[${this.ui.colour}]`,
      !hasBorder || !this.noContent
        ? `border-b border-[${this.ui.borderColour}]`
        : '',
      `text-[${this.ui.textColour}]`
    )

    return html`<div class=${bodyStyles}>
      <slot name="body"></slot>
    </div>`
  }

  protected override renderContent() {
    const contentStyles = apply(
      'transition-[padding] ease-in-out duration-[250ms]',
      'h-full',
      this.toggle && !(this.noContent || this.noContentPadding)
        ? this.depth == 0
          ? // For top level node cars use larger left margin so that
            // node chips in content are visible
            'py-3 pl-14 pr-4'
          : 'p-3'
        : ''
    )

    return html`
      <div class=${!this.displayContent && this.toggle ? 'hidden h-full' : 'block h-full'}>
        ${this.renderMarker()}
        <div class="${contentStyles}">
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
