import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { getTitleIcon } from '../ui/nodes/properties/programming-language'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { CodeStatic } from './code-static'

@customElement('stencila-code-block')
export class CodeBlock extends CodeStatic {
  override render() {
    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: 'Code Block',
      icon: 'code',
    }

    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    return html`
      <stencila-ui-block-on-demand
        type="CodeBlock"
        node-id=${this.id}
        depth=${this.depth}
        header-icon=${icon}
        header-title=${title}
      >
        <div slot="header-right">
          <stencila-ui-node-chat-commands
            type="CodeBlock"
            node-id=${this.id}
            depth=${this.depth}
          >
          </stencila-ui-node-chat-commands>
        </div>

        <div slot="body">
          <stencila-ui-node-authors type="CodeBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>

        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderContent() {
    return html`
      <div class="my-2">
        <stencila-ui-node-code
          type="CodeBlock"
          code=${this.code}
          .code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          read-only
          no-gutters
          container-classes=${`rounded-sm border border-gray-200`}
        >
        </stencila-ui-node-code>
      </div>
    `
  }
}
