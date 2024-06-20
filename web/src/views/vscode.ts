import { InlineTypeList } from '@stencila/types'
import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { Entity } from '../nodes/entity'
import { DocumentPreviewBase } from '../ui/nodes/mixins/preview-base'
import { ChipToggleInterface } from '../ui/nodes/mixins/toggle-chip'
import { UIBaseClass } from '../ui/nodes/mixins/ui-base-class'

import '../nodes'
import '../shoelace'
import '../ui/preview-menu'

type MessagePayload = {
  // the id of the node to scroll the view to
  scrollTarget?: string
}

/**
 * A view for a VSCode WebView preview panel
 *
 * This will use message passing and `morphdom` to update the content.
 */
@customElement('stencila-vscode-view')
export class VsCodeView extends DocumentPreviewBase {
  protected override createRenderRoot(): this {
    const lightDom = super.createRenderRoot()

    /**
     * must be declared here as an arrow func,
     * in order to bind `this` to the function.
     */
    const handleMessages = (e: Event & { data: MessagePayload }) => {
      const { scrollTarget } = e.data
      if (scrollTarget) {
        const targetEl = this.querySelector(`#${e.data.scrollTarget}`) as Entity

        if (targetEl) {
          targetEl.scrollIntoView({
            block: 'start',
            inline: 'nearest',
            behavior: 'smooth',
          })

          let cardType = 'stencila-ui-block-on-demand'

          if (InlineTypeList.includes(scrollTarget.constructor.name)) {
            cardType = 'stencila-ui-inline-on-demand'
          }
          const card = targetEl.shadowRoot.querySelector(
            cardType
          ) as UIBaseClass & ChipToggleInterface

          if (card) {
            card.openCard()
          }
        }
      }
    }

    // add message event listener for messages from the vscode window
    window.addEventListener('message', handleMessages)

    return lightDom
  }

  protected override render() {
    return html`
      <slot></slot>
      ${this.renderPreviewMenu()}
    `
  }
}
