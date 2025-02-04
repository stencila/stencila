import { LabelType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { getTitleIcon } from '../ui/nodes/properties/programming-language'
import { booleanConverter } from '../utilities/booleanConverter'

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
    converter: booleanConverter,
  })
  isInvisible?: boolean

  @state()
  hasOutputs: boolean = false

  protected handleOutputsChange(e: Event) {
    const slot = e.target as HTMLSlotElement
    this.hasOutputs = !!slot.assignedElements()[0]
  }

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

  renderCard() {
    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: 'Code Chunk',
      icon: 'code',
    }

    const readOnly = ['Running', 'Pending'].includes(this.executionStatus)

    const hasDocRoot = this.hasDocumentRootNode()

    return html`
      <stencila-ui-block-on-demand
        type="CodeChunk"
        node-id=${this.id}
        depth=${this.depth}
        header-icon=${icon}
        header-title=${title}
        ?noVisibleContent=${!this.hasOutputs}
        ?no-root=${!hasDocRoot}
      >
        <span slot="header-right" class="flex flex-row items-center gap-3">
          <stencila-ui-node-execution-commands
            type="CodeChunk"
            node-id=${this.id}
            depth=${this.depth}
            status=${this.executionStatus}
            required=${this.executionRequired}
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
            node-id=${this.id}
            code=${this.code}
            code-authorship=${this.codeAuthorship}
            language=${this.programmingLanguage}
            execution-required=${this.executionRequired}
            ?read-only=${readOnly}
          >
            <slot name="execution-messages" slot="messages"></slot>
          </stencila-ui-node-code>
        </div>
        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
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
          <slot name="outputs" @slotchange=${this.handleOutputsChange}></slot>
          ${this.labelType === 'FigureLabel'
            ? html`<figcaption><slot name="caption"></slot></figcaption>`
            : ''}
        `
  }
}
