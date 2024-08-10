import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { getTitleIcon } from '../ui/nodes/properties/programming-language'
import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance/provenance'

import { CodeStatic } from './code-static'

@customElement('stencila-code-block')
export class CodeBlock extends CodeStatic {
  override render() {
    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: 'CodeChunk',
      icon: 'code',
    }

    return html`
      <stencila-ui-block-on-demand
        type="CodeBlock"
        programming-language=${this.programmingLanguage}
        depth=${this.depth}
        ancestors=${this.ancestors}
        header-icon=${icon}
        header-title=${title}
      >
        <div slot="body">
          <stencila-ui-node-authors type="CodeBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
        <div slot="content" class="my-2">
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
      </stencila-ui-block-on-demand>
    `
  }
}
