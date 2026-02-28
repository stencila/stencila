import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../twind'
import { iconMaybe } from '../ui/icons/icon'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Supplement`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/supplement.md
 */
@customElement('stencila-supplement')
@withTwind()
export class Supplement extends Entity {
  @property({ attribute: 'work-type' })
  workType?: string

  @property()
  target?: string

  /**
   * Toggle show/hide work content
   *
   * Defaults to false (hidden), and then is toggled on/off by user.
   */
  @state()
  private showWork?: boolean = false

  /**
   * Whether the supplement has any work content
   *
   * Used to determine whether to show the chevron button or external link.
   */
  @state()
  private hasWork = false

  /**
   * A mutation observer to update the `hasWork` state when
   * the `work` slot changes
   */
  private workObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `work` slot
   */
  private onWorkSlotChange(event: Event) {
    // Get the slot element
    const workElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    // Set current state
    this.hasWork = workElem && workElem.childElementCount > 0

    // Update the state when the slot is mutated
    if (this.workObserver) {
      this.workObserver.disconnect()
    }
    if (workElem) {
      this.workObserver = new MutationObserver(() => {
        this.hasWork = workElem.childElementCount > 0
      })
      this.workObserver.observe(workElem, {
        childList: true,
      })
    }
  }

  override render() {
    if (this.isWithin('StyledBlock')) {
      return this.renderContent()
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithChatAction()
    }

    return this.renderCard()
  }

  private renderContent() {
    return html`
      <slot name="id"></slot>

      <div class="flex items-end justify-between items-center">
        <slot name="caption"></slot>

        ${this.hasWork
          ? html`<stencila-ui-chevron-button
              default-pos=${this.showWork ? 'down' : 'left'}
              custom-class="flex items-center"
              .clickEvent=${(e: Event) => {
                e.stopImmediatePropagation()
                this.showWork = !this.showWork
              }}
            ></stencila-ui-chevron-button>`
          : html`<div class="text-sm text-blue-700">
              <a href=${this.target} target="_blank"><stencila-ui-icon name="externalLink"></stencila-ui-icon></a>
            </div>`}
      </div>

      <stencila-ui-collapsible-animation
        class=${this.showWork ? 'opened' : ''}
      >
        <slot name="work" @slotchange=${this.onWorkSlotChange}></slot>
      </stencila-ui-collapsible-animation>
    `
  }

  override renderCard() {
    const icon = this.workType ? nodeUi(this.workType as NodeType).icon : iconMaybe('file')
    const title = `Supplementary ${this.workType?.replace(/Object$/, '') ?? 'Material'}`

    return html`
      <stencila-ui-block-on-demand
        type=${this.type()}
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
        header-icon=${icon}
        header-title=${title}
      >
        <div slot="content">
          ${this.renderContent()}
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
