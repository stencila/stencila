import { LabelType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, query, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { getTitleIcon } from '../ui/nodes/properties/programming-language'

import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/label-and-caption'
import '../ui/nodes/properties/provenance'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `CodeChunk` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-chunk.md
 */
@customElement('stencila-code-chunk')
@withTwind()
export class CodeChunk extends CodeExecutable {
  @property({ attribute: 'label-type' })
  labelType?: LabelType

  @property()
  label?: string

  @property({
    attribute: 'is-invisible',
    type: Boolean,
    // Converter needed because encoded not a boolean attribute (present or absent)
    // but as a stringified boolean
    converter: (attr) => attr == 'true',
  })
  isInvisible: boolean = false

  @query('slot[name="outputs"]')
  outputsSlot!: HTMLSlotElement

  @state()
  hasOutputs: boolean = false

  protected handleOutputsChange() {
    this.hasOutputs = !!this.outputsSlot.assignedElements()[0]
  }

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: 'Code Chunk',
      icon: 'code',
    }

    return html`<stencila-ui-block-on-demand
      type="CodeChunk"
      node-id=${this.id}
      depth=${this.depth}
      header-icon=${icon}
      header-title=${title}
      ?noVisibleContent=${!this.hasOutputs}
    >
      <span slot="header-right" class="flex flex-row items-center gap-3">
        <stencila-ui-node-chat-commands
          type="CodeChunk"
          node-id=${this.id}
          depth=${this.depth}
        >
        </stencila-ui-node-chat-commands>

        <stencila-ui-node-execution-commands
          type="CodeChunk"
          node-id=${this.id}
          depth=${this.depth}
        >
        </stencila-ui-node-execution-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="CodeChunk"
          node-id=${this.id}
          mode=${this.executionMode}
          bounds=${this.executionBounds}
          .tags=${this.executionTags}
          status=${this.executionStatus}
          required=${this.executionRequired}
          count=${this.executionCount}
          bounded=${this.executionBounded}
          ended=${this.executionEnded}
          duration=${this.executionDuration}
        >
          <slot name="execution-dependencies"></slot>
          <slot name="execution-dependants"></slot>
        </stencila-ui-node-execution-details>

        <stencila-ui-node-authors type="CodeChunk">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="CodeChunk"
          code=${this.code}
          code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          execution-required=${this.executionRequired}
          read-only
        >
          <slot name="execution-messages" slot="messages"></slot>
        </stencila-ui-node-code>
      </div>

      <div slot="content">${this.renderContent()}</div>
    </stencila-ui-block-on-demand>`
  }

  /*
  private renderShowHideOutput() {
    return html`<sl-tooltip
      content=${this.isInvisible ? 'Show output' : 'Hide output'}
    >
      <stencila-ui-icon-button
        class="text-xl"
        name=${this.isInvisible ? 'eye' : 'eyeSlash'}
        @click=${(e: Event) => {
          // Stop the click behavior of the card header parent element
          e.stopImmediatePropagation()
          this.isInvisible = !this.isInvisible
        }}
      ></stencila-ui-icon-button>
    </sl-tooltip>`
  }
  */

  private renderContent() {
    return this.isInvisible
      ? html``
      : html`
          ${this.labelType === 'TableLabel'
            ? html`<caption class="block">
                <slot name="caption"></slot>
              </caption>`
            : ''}
          <slot name="outputs"></slot>
          ${this.labelType === 'FigureLabel'
            ? html`<figcaption><slot name="caption"></slot></figcaption>`
            : ''}
        `
  }
}
