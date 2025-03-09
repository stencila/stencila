import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import { getTitleIcon } from '../ui/nodes/properties/programming-language'

import { Entity } from './entity'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

@customElement('stencila-raw-block')
@withTwind()
export class RawBlock extends Entity {
  @property()
  format: string

  @property()
  content: string

  @property({ attribute: 'content-authorship' })
  contentAuthorship?: string

  private resetContentCss(e: Event) {
    const slotted = (e.target as HTMLSlotElement).assignedElements()[0]
    if (slotted) {
      slotted.classList.add('not-prose')
    }
  }

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`<slot name="content"></slot>`
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithChatAction()
    }

    return this.renderCard()
  }

  override renderCard() {
    const { title, icon } = getTitleIcon(this.format) ?? {
      title: this.format,
      icon: 'fileTypeRaw',
    }

    return html`
      <stencila-ui-block-on-demand
        type="RawBlock"
        node-id=${this.id}
        depth=${this.depth}
        header-icon=${icon}
        header-title="Raw ${title}"
      >
        <div slot="body">
          <stencila-ui-node-authors type="RawBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="RawBlock"
            node-id=${this.id}
            node-property="content"
            code=${this.content}
            .code-authorship=${this.contentAuthorship}
            language=${this.format}
            ?read-only=${!this.hasDocumentRootNode()}
          >
            <slot name="compilation-messages" slot="messages"></slot>
          </stencila-ui-node-code>
        </div>

        <div slot="content">
          <slot name="content" @slotchange=${this.resetContentCss}></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
