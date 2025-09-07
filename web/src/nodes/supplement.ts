import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'
import { iconMaybe } from '../ui/icons/icon'

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

  /**
   * Toggle show/hide work content
   *
   * Defaults to false (hidden), and then is toggled on/off by user.
   */
  @state()
  private showWork?: boolean = false

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

      <div class="flex items-end justify-between">
        <slot name="caption"></slot>
        <stencila-ui-chevron-button
          default-pos=${this.showWork ? 'down' : 'left'}
          custom-class="flex items-center"
          .clickEvent=${(e: Event) => {
            e.stopImmediatePropagation()
            this.showWork = !this.showWork
          }}
        ></stencila-ui-chevron-button>
      </div>

      <stencila-ui-collapsible-animation
        class=${this.showWork ? 'opened' : ''}
      >
        <slot name="work"></slot>
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
