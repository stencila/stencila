import { AdmonitionType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../twind'
import { admonitionUi } from '../ui/nodes/icons-and-colours'
import { booleanConverter } from '../utilities/booleanConverter'

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
  @property({ attribute: 'is-folded', converter: booleanConverter })
  isFolded?: boolean

  /**
   * Whether the admonition has a title.
   *
   * Used to generate a title using the `admonitionType` if necessary.
   */
  @state()
  hasTitleSlot: boolean

  /**
   * The text of the title.
   *
   * Used to avoid adding an insert chip for some types of admonitions.
   */
  private titleSlotText?: string

  private toggleIsFolded() {
    if (this.isFolded) {
      this.isFolded = false
    } else {
      this.isFolded = true
    }
  }

  private onTitleSlotChange(event: Event): void {
    const slot = event.target as HTMLSlotElement
    this.hasTitleSlot = slot.assignedElements().length > 0
    this.titleSlotText = this.hasTitleSlot
      ? slot.assignedElements()[0].textContent
      : undefined
  }

  override render() {
    const { borderColour } = admonitionUi(this.admonitionType)

    const styles = apply([
      'my-4',
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

    // Render with an insert chip when in model chat response but not if
    // a "Thinking" admonition
    if (this.isWithinModelChatMessage()) {
      return html`
        <div class="group relative">
          ${this.titleSlotText !== 'Thinking' ? this.renderInsertChip() : ''}
          <div class=${styles}>
            ${this.renderHeader()} ${this.renderContent()}
          </div>
        </div>
      `
    }
    const hasDocRoot = this.hasDocumentRootNode()

    return html`
      <stencila-ui-block-on-demand
        type="Admonition"
        node-id=${this.id}
        depth=${this.depth}
        ?no-root=${!hasDocRoot}
      >
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
      `${this.isFolded ? 'rounded-r' : 'rounded-tr'}`,
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
              default-pos=${this.isFolded ? 'left' : 'down'}
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
      this.isFolded ? 'opacity-0' : 'opacity-100',
      this.isFolded ? 'max-h-0' : 'max-h-[10000px]',
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
