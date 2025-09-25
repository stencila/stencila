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
    attribute: 'is-echoed',
    converter: booleanConverter,
  })
  isEchoed?: boolean

  @property({
    attribute: 'is-hidden',
    converter: booleanConverter,
  })
  isHidden?: boolean

  @state()
  hasOutputs: boolean = false

  protected handleOutputsChange(e: Event) {
    const slot = e.target as HTMLSlotElement
    this.hasOutputs = !!slot.assignedElements()[0]
  }

  override render() {
    if (this.isWithin('StyledBlock')) {
      return this.renderOutputs()
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithChatAction()
    }

    return this.renderCard()
  }

  override renderCard() {
    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: this.programmingLanguage ?? 'Code Chunk',
      icon: 'code',
    }

    const hasRoot = this.hasRoot()

    const readOnly =
      ['Running', 'Pending'].includes(this.executionStatus) || !hasRoot

    return html`
      <stencila-ui-block-on-demand
        type="CodeChunk"
        node-id=${this.id}
        depth=${this.depth}
        header-icon=${icon}
        header-title=${title}
        ?no-content-padding=${!this.hasOutputs || this.isHidden}
        ?has-root=${hasRoot}
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
            is-echoed=${this.isEchoed !== undefined
              ? this.isEchoed.toString()
              : undefined}
            is-hidden=${this.isHidden !== undefined
              ? this.isHidden.toString()
              : undefined}
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
            <slot name="compilation-messages" slot="messages"></slot>
            <slot name="execution-messages" slot="messages"></slot>
          </stencila-ui-node-code>
        </div>

        <div slot="content">
          <slot name="id"></slot>
          ${this.isEchoed && !this.context.cardOpen ? this.renderCode() : ''}
          ${this.isHidden ? '' : this.renderOutputs()}
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  private renderCode() {
    return html`<div class="my-2">
      <stencila-ui-node-code
        type="CodeChunk"
        node-id=${this.id}
        code=${this.code}
        code-authorship=${this.codeAuthorship}
        language=${this.programmingLanguage}
        execution-required=${this.executionRequired}
        read-only
        no-gutters
      >
        <slot name="compilation-messages" slot="messages"></slot>
        <slot name="execution-messages" slot="messages"></slot>
      </stencila-ui-node-code>
    </div>`
  }

  private renderOutputs() {
    return html`
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
