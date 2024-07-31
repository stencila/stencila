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
      `text-[${this.ui.textColour}]`,
      'my-2',
      this.ui.borderColour && `border-[${this.ui.borderColour}]`,
    ])

    const headerStyles = this.collapsed && 'rounded-sm'

    return html`
      <div class=${`${cardStyles}`}>
        <div class="relative">
          ${this.renderHeader(headerStyles)} ${this.renderAnimatedCardBody()}
        </div>
        <div>${this.renderContent()}</div>
      </div>
    `
  }

  protected override renderBody() {
    const { colour } = this.ui
    const bodyStyles = apply(['relative', 'w-full h-full', `bg-[${colour}]`])

    return html`
      <div class=${bodyStyles}>
        <slot name="body"></slot>
      </div>
    `
  }

  protected override renderContent() {
    const contentStyles = apply([
      'flex',
      'relative',
      'transition-[padding] ease-in-out duration-[250ms]',
      'px-3',
    ])

    return html`
      <div class=${!this.displayContent ? 'hidden' : 'block'}>
        <div class=${contentStyles}>
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
