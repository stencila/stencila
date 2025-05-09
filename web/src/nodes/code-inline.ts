import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { getTitleIcon } from '../ui/nodes/properties/programming-language'

import '../ui/nodes/cards/block-in-flow'
import '../ui/nodes/cards/inline-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { CodeStatic } from './code-static'

@customElement('stencila-code-inline')
@withTwind()
export class CodeInline extends CodeStatic {
  override render() {
    // If no programming language, then do not show node card to reduce visual noise
    if (!this.programmingLanguage || this.isWithinUserChatMessage()) {
      return html`<slot></slot>`
    }

    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: 'Inline Code',
      icon: 'code',
    }

    return html`
      <stencila-ui-inline-on-demand
        type="CodeInline"
        header-icon=${icon}
        header-title=${title}
      >
        <div slot="body">
          <stencila-ui-node-authors type="CodeInline">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="CodeInline"
            node-id=${this.id}
            code=${this.code}
            .code-authorship=${this.codeAuthorship}
            language=${this.programmingLanguage}
            read-only
          >
          </stencila-ui-node-code>
        </div>
        <span slot="content">
          <slot></slot>
        </span>
      </stencila-ui-inline-on-demand>
    `
  }
}
