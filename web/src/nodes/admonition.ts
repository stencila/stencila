import { AdmonitionType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../twind'
import { admonitionUi } from '../ui/nodes/icons-and-colours'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Admonition` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/admonition.md
 */
@customElement('stencila-admonition')
@withTwind()
export class Admonition extends Entity {
  /**
   * The type of admonition.
   */
  @property({ attribute: 'admonition-type' })
  admonitionType: AdmonitionType

  /**
   * Whether the admonition is folded.
   */
  @property({ attribute: 'is-folded' })
  isFolded?: 'true' | 'false'

  /**
   * Whether the admonition has a title.
   *
   * Used to generate a title using the `admonitionType` if necessary.
   */
  @state()
  hasTitleSlot: boolean

  private toggleIsFolded() {
    if (this.isFolded === 'true') {
      this.isFolded = 'false'
    } else {
      this.isFolded = 'true'
    }
  }

  private onTitleSlotChange(event: Event): void {
    const slot = event.target as HTMLSlotElement
    this.hasTitleSlot = slot.assignedElements().length > 0
  }

  override render() {
    const { borderColour } = admonitionUi(this.admonitionType)

    const styles = apply([
      `border border-l-4 border-[${borderColour}]`,
      'shadow rounded',
    ])

    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`
        <div class=${styles}>
          ${this.renderHeader()} ${this.renderContent()}
        </div>
      `
    }

    return html`
      <stencila-ui-block-on-demand
        type="Admonition"
        node-id=${this.id}
        depth=${this.depth}
      >
        <div slot="header-right">
          <stencila-ui-node-chat-commands
            type="Admonition"
            node-id=${this.id}
            depth=${this.depth}
          >
          </stencila-ui-node-chat-commands>
        </div>

        <div slot="body">
          <stencila-ui-node-authors type="Admonition">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>

        <div slot="content" class="mt-2">
          <div class=${styles}>
            ${this.renderHeader()} ${this.renderContent()}
          </div>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  protected renderHeader() {
    const { textColour, baseColour, icon } = admonitionUi(this.admonitionType)

    const styles = apply([
      'flex items-center',
      'p-2',
      `text-[${textColour}]`,
      `bg-[${baseColour}]`,
      `${this.isFolded === 'true' ? 'rounded-r' : 'rounded-tr'}`,
    ])

    return html`
      <div class=${styles}>
        <stencila-ui-icon name=${icon}> </stencila-ui-icon>
        <div class="ml-2 flex-grow text-sm font-semibold">
          <slot name="title" @slotchange=${this.onTitleSlotChange}></slot>
          ${!this.hasTitleSlot ? this.admonitionType : ''}
        </div>

        ${this.isFolded !== undefined
          ? html`<stencila-ui-chevron-button
              default-pos=${this.isFolded === 'true' ? 'left' : 'down'}
              slot="right-side"
              custom-class="flex items-center"
              .clickEvent=${() => this.toggleIsFolded()}
            ></stencila-ui-chevron-button>`
          : ''}
      </div>
    `
  }

  protected renderContent() {
    const styles = apply([
      this.isFolded === 'true' ? 'opacity-0' : 'opacity-100',
      this.isFolded === 'true' ? 'max-h-0' : 'max-h-[10000px]',
      'transition-all duration-200',
    ])

    return html`
      <div class=${styles}>
        <div class="p-2">
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
