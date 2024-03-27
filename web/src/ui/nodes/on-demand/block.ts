import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { PropertyValueMap, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import '../../animation/collapsible'
import { UIBaseClass } from '../mixins/uiBaseClass'

/**
 * UI block-on-demand
 *
 * A component to render a node-card on demand - i.e. a user requests to see
 * the info rather than just the content of a card.
 */
@customElement('stencila-ui-block-on-demand')
@withTwind()
export class UIBlockOnDemand extends UIBaseClass {
  /**
   * Manages showing/hiding the card info (when rendering display = 'on-demand')
   */
  @state()
  toggle: boolean = false

  /**
   * Disables showing content if slot has no content.
   */
  @state()
  displayContent: boolean = false

  override render() {
    const cardStyles = apply([
      'group',
      'transition duration-400',
      'border border-[transparent]',
      'rounded',
      this.view === 'source' ? 'flex flex-col h-full' : 'my-2',
      this.toggle && `border-[${this.ui.borderColour}]`,
    ])

    return html`<div class=${`${cardStyles}`}>
      <div class="relative">
        <stencila-ui-collapsible-animation class=${this.toggle ? 'opened' : ''}>
          ${this.renderHeader()} ${this.renderBody()}
        </stencila-ui-collapsible-animation>
        ${this.renderContent()}
      </div>
    </div>`
  }

  private renderHeader() {
    const { iconLibrary, icon, title, borderColour } = this.ui

    const headerStyles = apply([
      'flex items-center',
      'w-full',
      'px-6 py-3',
      'gap-x-2',
      `bg-[${borderColour}]`,
      `border border-[${borderColour}]`,
      this.view === 'source' ? '' : 'rounded-t',
      'font-medium',
    ])

    return html`<div class=${headerStyles}>
      <div class="flex items-center gap-x-2 grow">
        ${this.renderClose()}
        <span class="items-center flex grow-0 shrink-0">
          <sl-icon
            library=${iconLibrary}
            name=${icon}
            class="text-2xl"
          ></sl-icon>
        </span>
        <div class="flex justify-between items-center gap-x-2 grow">
          <span class="font-bold grow">${title}</span>
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
      `border border-[${borderColour}] rounded-b`,
    ])

    return html`<div class=${bodyStyles}>
      <slot name="body"></slot>
    </div>`
  }

  private renderClose() {
    const styles = apply(['text-base', 'cursor-pointer', 'grow-0 shrink-0'])

    return html`<sl-icon
      class=${styles}
      name="chevron-down"
      library="default"
      @click=${this.toggleCardDisplay}
    ></sl-icon>`
  }

  private renderChip() {
    const { iconLibrary, icon, colour, borderColour } = this.ui

    const styles = apply([
      this.toggle && 'pointer-events-none',
      !this.toggle && 'group-hover:opacity-100',
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
          <sl-icon
            library=${iconLibrary}
            name=${icon}
            class="text-base"
          ></sl-icon>
        </div>
      </div>
    `
  }

  private renderContent() {
    const contentStyles = apply([
      !this.displayContent && this.toggle ? 'hidden' : 'flex',
      'relative',
      'transition-[padding] ease-in-out duration-[250ms]',
      'px-0',
      this.toggle && 'px-3',
    ])

    return html` <div class=${contentStyles}>
      ${this.renderChip()}
      <slot name="content"></slot>
    </div>`
  }

  private toggleCardDisplay() {
    this.toggle = !this.toggle
  }

  protected override update(
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    changedProperties: PropertyValueMap<any> | Map<PropertyKey, unknown>
  ) {
    super.update(changedProperties)
    const slot: HTMLSlotElement = this.shadowRoot.querySelector(
      'slot[name="content"]'
    )

    if (slot) {
      const hasItems = slot.assignedElements({ flatten: true }).length !== 0

      if (hasItems !== this.displayContent) {
        this.displayContent = hasItems
      }
    }
  }
}
