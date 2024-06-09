import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply, css as twCss, Twind } from '@twind/core'
import { css, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../../../../twind'
import '../../../animation/collapsible'
import { ToggleChipMixin } from '../../mixins/toggle-chip'
import { UIBaseCard } from '../base-card'

/**
 * UI inline-on-demand
 *
 * A component to render a node-card on demand - i.e. a user requests to see
 * the info rather than just the content of a card.
 */
@customElement('stencila-ui-inline-on-demand')
@withTwind()
export class UIInlineOnDemand extends ToggleChipMixin(UIBaseCard) {
  @state()
  isToolTipOpen: boolean = false

  protected override restrictTitleWidth: boolean = true

  protected override toggleChipPosition: string = '-top-1/2 absolute'

  private tw: Twind

  static override styles = css`
    :host {
      display: inline-block;
    }
  `

  override render() {
    const cardStyles = apply([
      'group',
      'transition duration-400',
      'rounded',
      this.view === 'source' ? 'flex flex-col h-full' : '',
    ])

    return html`<div class=${`ui-inline-on-demand ${cardStyles}`}>
      ${this.renderContentContainer()}
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

  private renderContentContainer() {
    const containerStyles = apply([
      !this.displayContent && this.toggle ? 'hidden' : 'flex',
      'relative',
      'transition-[padding] ease-in-out duration-[250ms]',
      'px-0',
    ])

    const css = twCss
    const colors = this.tw.theme().colors

    const toolTipStyles = css`
      &::part(body) {
        --max-width: 24rem;
        color: ${(colors['black'] ?? 'black') as string};
        padding: 0;
        border-radius: 0;
        background-color: transparent;
        pointer-events: all;
      }

      &::part(base__arrow) {
        display: none;
      }

      &::part(body)::after {
        box-shadow: 0 0 10px rgba(0, 0, 0, 0.15);
        mix-blend-mode: multiply;
        content: '';

        position: absolute;
        top: 0;
        right: 0;
        left: 0;
        bottom: 0;
        z-index: -1;
      }
    `

    return html`<div class=${containerStyles}>
      ${this.renderChip(this.getIcon(), this.ui)}
      <sl-tooltip
        trigger="manual"
        class=${`${toolTipStyles}`}
        .open=${this.toggle}
        placement="bottom"
      >
        <div slot="content">
          ${this.renderHeader()} ${this.renderAnimatedContent()}
        </div>
        <div class="inline">
          <slot name="content"></slot>
        </div>
      </sl-tooltip>
    </div>`
  }

  protected override toggleCardDisplay() {
    this.toggle = !this.toggle

    this.shadowRoot.dispatchEvent(
      new CustomEvent(`toggle-${this.id}`, {
        bubbles: true,
        composed: true,
        detail: { cardOpen: this.toggle, nodeId: this.nodeId },
      })
    )
  }
}
