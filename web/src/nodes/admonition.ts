import { AdmonitionType } from '@stencila/types'
import { apply } from '@twind/core'
import { PropertyValueMap, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../twind'
import { admonitionUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

import '../ui/nodes/node-card/on-demand/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance/provenance'

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

  @state()
  hasTitleSlot: boolean

  private toggleIsFolded() {
    if (this.isFolded === 'true') {
      this.isFolded = 'false'
    } else {
      this.isFolded = 'true'
    }
  }

  protected override firstUpdated(
    _changedProperties: PropertyValueMap<this> | Map<PropertyKey, unknown>
  ): void {
    super.firstUpdated(_changedProperties)

    const slot = this.shadowRoot.querySelector(
      'slot[name="title"]'
    ) as HTMLSlotElement

    if (slot) {
      this.hasTitleSlot = slot.assignedElements().length > 0
      slot.addEventListener('slotchange', () => {
        this.hasTitleSlot = slot.assignedElements().length > 0
      })
    }
  }

  override render() {
    const { borderColour } = admonitionUi(this.admonitionType)

    const styles = apply([
      `border border-l-4 border-[${borderColour}]`,
      'shadow rounded',
    ])

    return html`
      <stencila-ui-block-on-demand
        type="Admonition"
        depth=${this.depth}
        ancestors=${this.ancestors}
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
    const { textColour, baseColour, icon, iconLibrary } = admonitionUi(
      this.admonitionType
    )

    const styles = apply([
      'flex items-center',
      'p-2',
      `text-[${textColour}]`,
      `bg-[${baseColour}]`,
      `${this.isFolded === 'true' ? 'rounded-r' : 'rounded-tr'}`,
    ])

    return html`
      <div class=${styles}>
        <sl-icon name=${icon} library=${iconLibrary}> </sl-icon>
        <div class="ml-2 flex-grow text-sm font-semibold">
          <slot name="title"></slot>
          ${
            // use `admonitionType` as default title
            !this.hasTitleSlot ? this.admonitionType : ''
          }
        </div>

        <!-- TODO: Chevron if this.isFolded is defined, downward if false, right if true -->
        ${this.isFolded !== undefined
          ? html`<stencila-chevron-button
              default-pos=${this.isFolded === 'true' ? 'left' : 'down'}
              slot="right-side"
              custom-class="flex items-center"
              .clickEvent=${() => {
                this.toggleIsFolded()
              }}
            ></stencila-chevron-button>`
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
