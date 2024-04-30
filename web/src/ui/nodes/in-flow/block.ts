import '@shoelace-style/shoelace/dist/components/icon/icon'
import { apply } from '@twind/core'
import { PropertyValueMap, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import '../../animation/collapsible'
import { UIBaseClass } from '../mixins/uiBaseClass'

/**
 * UI in-flow-block
 *
 * A component to render a node-card "in flow" - i.e. renders a block as is
 * without requiring user interaction
 */
@customElement('stencila-ui-block-in-flow')
@withTwind()
export class UIBlockInFlow extends UIBaseClass {
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
      this.ui.borderColour && `border-[${this.ui.borderColour}]`,
    ])

    return html`<div class=${`${cardStyles}`}>
      <div class="relative">
        <stencila-ui-collapsible-animation class=${'opened'}>
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
      'px-4 py-2',
      'gap-x-2',
      `bg-[${borderColour}]`,
      `border border-[${borderColour}]`,
      this.view === 'source' ? '' : 'rounded-t',
      'font-medium',
    ])

    return html`<div class=${headerStyles}>
      <div class="flex items-center gap-x-2 grow">
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

  private renderContent() {
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
