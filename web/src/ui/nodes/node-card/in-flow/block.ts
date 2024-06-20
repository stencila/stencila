import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../../twind'
import { UIBaseCard } from '../base-card'

/**
 * UI in-flow-block
 *
 * A component to render a node-card "in flow" - i.e. renders a block as is
 * without requiring user interaction
 */
@customElement('stencila-ui-block-in-flow')
@withTwind()
export class UIBlockInFlow extends UIBaseCard {
  override render() {
    const cardStyles = apply([
      'group',
      'transition duration-400',
      'border border-[rgba(255,255,255,0)]',
      'rounded',
      this.view === 'source' ? 'flex flex-col h-full' : 'my-2',
      this.ui.borderColour && `border-[${this.ui.borderColour}]`,
    ])

    return html`<div class=${`${cardStyles}`}>
      <div class="relative">
        ${this.renderHeader()} ${this.renderAnimatedCardBody()}
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
      'flex',
      'relative',
      'transition-[padding] ease-in-out duration-[250ms]',
      'px-3',
    ])

    return html`<div class=${contentStyles}>
      <slot name="content"></slot>
    </div>`
  }
}
