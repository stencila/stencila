import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply, css as twCss, Twind } from '@twind/core'
import { PropertyValueMap, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import '../../animation/collapsible'
import { UIBaseClass } from '../mixins/uiBaseClass'

/**
 * UI inline-on-demand
 *
 * A component to render a node-card on demand - i.e. a user requests to see
 * the info rather than just the content of a card.
 */
@customElement('stencila-ui-inline-on-demand')
@withTwind()
export class UIInlineOnDemand extends UIBaseClass {
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

  @state()
  isToolTipOpen: boolean = false

  private tw: Twind

  override render() {
    const cardStyles = apply([
      'group',
      'transition duration-400',
      'rounded',
      this.view === 'source' ? 'flex flex-col h-full' : 'my-2',
    ])

    return html`<div class=${`${cardStyles}`}>
      ${this.renderContentContainer()}
    </div>`
  }

  private renderHeader() {
    const { iconLibrary, icon, title, borderColour } = this.ui

    const headerStyles = apply([
      'flex items-center',
      'w-full',
      'px-4 py-1',
      'gap-x-2',
      `bg-[${borderColour}]`,
      'border-b border-black/20',
      'rounded-t',
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
      name="chevron-up"
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
      <div class="-ml-[40px] pr-[6px] top-1/2 -translate-y-1/2 absolute">
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
        --sl-tooltip-padding: 0;
        --sl-tooltip-border-radius: 0;
        --sl-tooltip-background-color: transparent;
        --sl-tooltip-color: ${(colors['black'] ?? 'black') as string};

        pointer-events: all;
      }
    `

    const contentStyles = apply([
      'inline-block',
      `bg-[${this.ui.borderColour}]`,
      'rounded-md',
      'cursor-default',
      `text-black leading-5`,
      'mb-auto mx-1 -mt-[0.125rem]',
      'py-[0.125rem] px-1.5',
    ])

    return html` <div
      class=${containerStyles}
      style="--sl-tooltip-arrow-size: 0;"
    >
      ${this.renderChip()}
      <sl-tooltip
        trigger="manual"
        class=${`${toolTipStyles}`}
        .open=${this.toggle}
        placement="bottom"
      >
        <div slot="content">${this.renderHeader()} ${this.renderBody()}</div>
        <div class=${contentStyles}>
          <slot name="content"></slot>
        </div>
      </sl-tooltip>
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
