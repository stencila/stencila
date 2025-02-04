import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { getTitleIcon } from '../ui/nodes/properties/programming-language'

import { CodeStatic } from './code-static'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

@customElement('stencila-code-block')
@withTwind()
export class CodeBlock extends CodeStatic {
  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    // render with the `insert` chip in model chat response
    if (this.isWithinModelChatMessage()) {
      return html`
        <div class="group relative">
          ${this.renderInsertChip()} ${this.renderCard()}
        </div>
      `
    }

    return this.renderCard()
  }

  private renderCard() {
    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: 'Code Block',
      icon: 'code',
    }

    const hasDocRoot = this.hasDocumentRootNode()

    return html`
      <stencila-ui-block-on-demand
        type="CodeBlock"
        node-id=${this.id}
        depth=${this.depth}
        header-icon=${icon}
        header-title=${title}
        ?no-root=${!hasDocRoot}
      >
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
          node-id=${this.id}
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
