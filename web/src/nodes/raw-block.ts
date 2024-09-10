import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { getTitleIcon } from '../ui/nodes/properties/programming-language'
import '../ui/nodes/node-card/in-flow/block'
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
    const { title, icon } = getTitleIcon(this.format) ?? {
      title: this.format,
      icon: 'fileTypeRaw',
    }

    return html`
      <stencila-ui-block-on-demand
        type="RawBlock"
        depth=${this.depth}
        ancestors=${this.ancestors}
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
            code=${this.content}
            .code-authorship=${this.contentAuthorship}
            language=${this.format}
            read-only
          >
          </stencila-ui-node-code>
        </div>

        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
