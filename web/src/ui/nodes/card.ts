import '@shoelace-style/shoelace/dist/components/icon/icon'
import type { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import { nodeUi } from './icons-and-colours'

import '../animation/collapsible'

/**
 * A component for displaying information about a node type (e.g. a `Heading` or `Table`)
 */
@customElement('stencila-ui-node-card')
@withTwind()
export class UINodeCard extends LitElement {
  /**
   * The type of node that this card is for
   *
   * Used to determine the title, icon and colors of the card.
   */
  @property()
  type: NodeType

  /**
   * Determine how to display the node-card. By default, we simply display the
   * card as is (`auto`). However, if we show the card in the dynamic view, we
   * need the ability to only show the card only when needed.
   */
  @property()
  display: 'on-demand' | 'auto' = 'auto'

  /**
   * Manages showing/hiding the card info (when rendering display = 'on-demand')
   */
  @state()
  toggle: boolean = false

  /**
   * If we encounter no content is in the slot, we need to hide the content area.
   */
  @state()
  showContent: boolean = true

  /**
   * Internal copy of the ui attributes.
   */
  private ui: ReturnType<typeof nodeUi> | undefined = undefined

  /**
   * Provide ui options based on the node type.
   */
  override connectedCallback() {
    super.connectedCallback()

    this.ui = nodeUi(this.type)
  }

  protected override firstUpdated() {
    const slot: HTMLSlotElement = this.renderRoot.querySelector(
      'slot[name="content"]'
    )

    if (slot) {
      this.showContent = slot.assignedElements({ flatten: true }).length !== 0
    }
  }

  override render() {
    const cardStyles = apply([
      'group',
      'transition duration-400',
      'border border-[transparent]',
      this.display && 'rounded',
      this.display === 'on-demand' &&
        this.toggle &&
        `border-[${this.ui.borderColour}]`,
    ])

    const contentStyles = apply([
      this.showContent ? 'flex' : 'hidden',
      'relative',
      'transition-[padding] ease-in-out duration-[250ms]',
      'px-0',
      'w-full',
      this.display === 'on-demand' && this.toggle && 'px-3',
    ])

    return html` <div class=${`${cardStyles}`}>
      <div class="relative">
        ${this.renderAnimation()}
        <div class=${contentStyles}>
          ${this.renderChip()}
          <div class="inline grow">
            <slot name="content"></slot>
          </div>
        </div>
      </div>
    </div>`
  }

  private renderAnimation() {
    if (this.display === 'on-demand') {
      return html`<stencila-ui-collapsible-animation
        class=${this.toggle ? 'opened' : ''}
      >
        ${this.renderHeader()} ${this.renderBody()}
      </stencila-ui-collapsible-animation>`
    }

    return html`${this.renderHeader()} ${this.renderBody()}`
  }

  private renderHeader() {
    const { icon, title, borderColour } = this.ui

    const headerStyles = apply([
      'flex items-center',
      'w-full',
      'px-4 py-1',
      'gap-x-2',
      `bg-[${borderColour}]`,
      `border border-[${borderColour}]`,
      'rounded-t',
      'font-sans',
      'font-medium',
    ])

    return html`<div class=${headerStyles}>
      <div class="flex items-center gap-x-2 grow">
        ${this.renderClose()}
        <span class="items-center flex grow-0 shrink-0">
          <stencila-ui-icon name=${icon} class="text-2xl"></stencila-ui-icon>
        </span>
        <div class="flex justify-between items-center gap-x-2 grow">
          <span class="font-semibold font-sans text-sm grow">${title}</span>
          <div class="">
            <slot name="header-right"></slot>
          </div>
        </div>
      </div>
    </div>`
  }

  private renderBody() {
    const { colour, borderColour } = this.ui
    const bodyStyles = apply([
      'relative',
      'w-full h-full',
      `bg-[${colour}]`,
      this.display === 'auto' && `border border-[${borderColour}] rounded-b`,
    ])

    return html`<div class=${bodyStyles}>
      <slot name="body"></slot>
    </div>`
  }

  private renderClose() {
    const styles = apply([
      'text-base',
      'cursor-pointer',
      'grow-0 shrink-0',
      this.display === 'auto' && 'hidden pointer-events-none',
    ])

    return html`<stencila-ui-icon
      class=${styles}
      name="chevronDown"
      @click=${this.toggleCardDisplay}
    ></stencila-ui-icon>`
  }

  private renderChip() {
    const { icon, colour, borderColour } = this.ui

    const styles = apply([
      this.display === 'auto' && `hidden pointer-events-none`,
      this.display === 'on-demand' && this.toggle && 'pointer-events-none',
      this.display === 'on-demand' && !this.toggle && 'group-hover:opacity-100',
      'h-8',
      'flex items-center',
      'opacity-0',
      'transition duration-200',
      'leading-none',
      'px-2 py-1.5',
      `bg-[${colour}]`,
      `border rounded-md border-[${borderColour}]`,
      'cursor-pointer',
      `fill-black text-black`,
      `hover:bg-[${borderColour}] hover:border-[${colour}]`,
    ])

    return html`
      <div class="-ml-[40px] pr-[6px] mt-2">
        <div class=${`${styles}`} @click=${this.toggleCardDisplay}>
          <stencila-ui-icon name=${icon} class="text-base"></stencila-ui-icon>
        </div>
      </div>
    `
  }

  private toggleCardDisplay() {
    this.toggle = !this.toggle
  }
}
