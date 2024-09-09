import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { IconName } from '../ui/icons/icon'
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
    let icon: IconName = 'fileTypeRaw'
    let title = `Raw ${this.format}`
    switch (this.format.toLowerCase()) {
      case 'css':
        icon = 'css'
        title = 'Raw CSS'
        break
      case 'js':
        icon = 'javascript'
        title = 'Raw JavaScript'
        break
      case 'html':
        icon = 'html'
        title = 'Raw HTML'
        break
      case 'md':
      case 'markdown':
        icon = 'markdown'
        title = 'Raw Markdown'
        break
      case 'svg':
        title = 'Raw SVG'
        break
    }

    return html`
      <stencila-ui-block-on-demand
        type="RawBlock"
        depth=${this.depth}
        ancestors=${this.ancestors}
        header-icon=${icon}
        header-title=${title}
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
