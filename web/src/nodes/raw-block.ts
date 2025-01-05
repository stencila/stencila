import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { getTitleIcon } from '../ui/nodes/properties/programming-language'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

@customElement('stencila-raw-block')
export class RawBlock extends Entity {
  @property()
  format: string

  @property()
  content: string

  @property({ attribute: 'content-authorship' })
  contentAuthorship?: string

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`<slot name="content"></slot>`
    }

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
        <div slot="header-right">
          <stencila-ui-node-chat-commands
            type="RawBlock"
            node-id=${this.id}
            depth=${this.depth}
          >
          </stencila-ui-node-chat-commands>
        </div>

        <div slot="body">
          <stencila-ui-node-authors type="RawBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="RawBlock"
            code=${this.content}
            .code-authorship=${this.contentAuthorship}
            language=${this.format}
            read-only
          >
            <slot name="compilation-messages" slot="messages"></slot>
          </stencila-ui-node-code>
        </div>

        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
