import { NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'
import { closestGlobally } from '../utilities/closestGlobally'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `Excerpt`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/excerpt.md
 */
@customElement('stencila-excerpt')
@withTwind()
export class Excerpt extends Entity {
  @property({ attribute: 'node-path' })
  nodePath: string

  @property({ attribute: 'node-ancestors' })
  nodeAncestors: string

  @property({ attribute: 'node-type' })
  nodeType_: NodeType

  /**
   * Toggle show/hide content
   *
   * Defaults to false, and then is toggled off/on by user.
   */
  @state()
  private showContent?: boolean = false

  public static shouldExpand(card: HTMLElement, nodeType: NodeType): boolean {
    return (
      nodeType == 'Excerpt' ||
      (['CodeChunk'].includes(nodeType) &&
        closestGlobally(card, 'stencila-excerpt') !== null)
    )
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
      <div>
        <slot name="content"></slot>
      </div>
    `
  }

  override renderCard() {
    return html`
      <stencila-ui-block-on-demand
        type="Excerpt"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
        no-content-padding
      >
        <div slot="header-right" class="flex items-center gap-1">
          <stencila-ui-icon
            name=${nodeUi(this.nodeType_).icon}
            class="text-sm"
          ></stencila-ui-icon>
          <span class="font-semibold text-sm">${this.nodeType_}</span>
        </div>
        <div slot="body" class="p-3">
          <slot name="source"></slot>
          <div class="flex items-center justify-between mt-2">
            ${this.renderAncestors()} ${this.renderShowHideContent()}
          </div>
        </div>

        <div
          slot="content"
          class="px-3 transition-[padding-top,padding-bottom] duration-500 ease-in-out ${this
            .showContent
            ? 'py-3'
            : 'py-0'}"
        >
          <stencila-ui-collapsible-animation
            class=${this.showContent ? 'opened' : ''}
          >
            ${this.renderContent()}
          </stencila-ui-collapsible-animation>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderAncestors() {
    return html`<div class="text-xs font-sans">
      ${this.nodeAncestors.replace(/\//g, ' > ')} > ${this.nodeType_}
    </div>`
  }

  private renderShowHideContent() {
    return html`<sl-tooltip
      content=${this.showContent
        ? 'Hide excerpt content'
        : 'Show excerpt content'}
    >
      <stencila-ui-icon-button
        class="text-sm"
        name=${this.showContent ? 'eyeSlash' : 'eye'}
        @click=${(e: Event) => {
          // Stop the click behavior of the card header parent element
          e.stopImmediatePropagation()
          this.showContent = !this.showContent
        }}
      ></stencila-ui-icon-button>
    </sl-tooltip>`
  }
}
