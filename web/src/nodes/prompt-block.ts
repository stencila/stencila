import { apply, css } from '@twind/core'
import { html, TemplateResult } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { documentCommandEvent } from '../clients/commands'
import { data, Prompt } from '../system'
import { withTwind } from '../twind'
import { iconMaybe } from '../ui/icons/icon'
import { NodeTypeUI, nodeUi } from '../ui/nodes/icons-and-colours'

import { Executable } from './executable'

import '../ui/nodes/properties/generic/collapsible-details'
import '../ui/nodes/cards/block-in-flow'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/provenance'

/**
 * Web component representing a Stencila Schema `PromptBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/prompt-block.md
 */
@customElement('stencila-prompt-block')
@withTwind()
export class PromptBlock extends Executable {
  @property()
  target: string

  /**
   * UI settings to use when rendering
   *
   * Instantiated in `connectedCallback` to avoid getting on each render.
   */
  private ui: NodeTypeUI

  /**
   * Prompt <select> options updated whenever prompt list is updated
   * rather than in `render()`
   */
  private promptOptions: TemplateResult[] = []

  /**
   * Toggle show/hide content
   *
   * Defaults to true, and then is toggled off/on by user.
   */
  @state()
  private showContent?: boolean = true

  /**
   * Determine if this is a property of another block
   */
  private isProperty(): boolean {
    return this.parentNodeIs('Chat') || this.parentNodeIs('InstructionBlock')
  }

  /**
   * On a change to the global list of models request an
   * update (re-render) of this component
   */
  private onPromptsUpdated() {
    // Group prompts by instruction type
    const types: Record<string, Prompt[]> = {}
    for (const prompt of data.prompts) {
      const type = prompt.instructionTypes[0] ?? 'Create'
      if (type in types) {
        types[type].push(prompt)
      } else {
        types[type] = [prompt]
      }
    }

    const { textColour } = this.ui

    // Pre-render options
    this.promptOptions = Object.entries(types).map(([type, prompts], index) => {
      return html`
        ${index !== 0 ? html`<sl-divider class="my-1"></sl-divider>` : ''}
        <div
          class="flex flex-row items-center gap-2 px-2 py-1 text-[${textColour}]"
        >
          <stencila-ui-icon
            slot="prefix"
            class="text-base"
            name=${iconMaybe(type.toLowerCase()) ?? 'box'}
          ></stencila-ui-icon>
          ${type}
        </div>
        ${prompts.map(
          (prompt) => html`
            <sl-option
              value=${prompt.id}
              style="--sl-spacing-x-small: 0.25rem;"
            >
              <span class="text-xs text-[${textColour}]"> ${prompt.id} </span>
            </sl-option>
          `
        )}
      `
    })

    this.requestUpdate()
  }

  /**
   * On a change to the `target` prompt property, send a patch to update it
   */
  private onPromptChanged(event: InputEvent) {
    let value = (event.target as HTMLInputElement).value

    if (value.trim().length === 0) {
      value = null
    }

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node',
        nodeType: 'PromptBlock',
        nodeIds: [this.id],
        nodeProperty: ['target', value],
      })
    )
  }

  override connectedCallback(): void {
    super.connectedCallback()

    this.ui = nodeUi(this.isProperty() ? this.parentNodeType : 'PromptBlock')

    this.showContent = !(
      this.parentNodeIs('Chat') || this.parentNodeIs('InstructionBlock')
    )

    data.addEventListener('prompts', this.onPromptsUpdated.bind(this))
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    data.removeEventListener('prompts', this.onPromptsUpdated.bind(this))
  }

  override render() {
    // Do not render in these contexts
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html``
    }

    const { colour, textColour, borderColour } = this.ui

    const headerClasses = apply(
      'flex flex-row items-center gap-x-2',
      'w-full',
      'px-3 py-1',
      `bg-[${colour}]`,
      `text-[${textColour}] text-xs leading-tight font-sans`,
      `border-t border-[${borderColour}]`
    )

    // Render as the property of a chat or instruction block
    if (this.isProperty()) {
      return html`
        <div class=${headerClasses}>
          Prompt ${this.renderPromptSelect(textColour)}
          ${this.renderShowHideContent()}
        </div>

        <div class="bg-white/50 ${this.showContent ? '' : 'hidden'}">
          ${this.renderContent()}
        </div>
      `
    }

    // Render as a standalone prompt preview
    return html`<stencila-ui-block-in-flow
      type="PromptBlock"
      node-id=${this.id}
      depth=${this.depth}
    >
      <span slot="header-right" class="flex flex-row items-center gap-x-3">
        <span class="text-sm"> ${this.target} </span>

        <stencila-ui-node-execution-commands
          type="PromptBlock"
          node-id=${this.id}
        >
        </stencila-ui-node-execution-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="PromptBlock"
          node-id=${this.id}
          mode=${this.executionMode}
          .tags=${this.executionTags}
          status=${this.executionStatus}
          required=${this.executionRequired}
          count=${this.executionCount}
          ended=${this.executionEnded}
          duration=${this.executionDuration}
        >
          <slot name="execution-dependencies"></slot>
          <slot name="execution-dependants"></slot>
        </stencila-ui-node-execution-details>

        <stencila-ui-node-execution-messages type="PromptBlock">
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <div class=${headerClasses}>
          Prompt ${this.renderPromptSelect(textColour)}
        </div>
      </div>

      <div slot="content" class="mx-auto">${this.renderContent()}</div>
    </stencila-ui-block-in-flow>`
  }

  private renderPromptSelect(textColour: string) {
    const style = css`
      &::part(display-input) {
        font-size: 0.75rem;
        color: ${textColour};
      }
      &::part(clear-button) {
        color: ${textColour};
      }
    `

    return html`<sl-select
      class="w-full ${style}"
      clearable
      size="small"
      value=${this.target}
      @sl-change=${(e: InputEvent) => this.onPromptChanged(e)}
    >
      ${this.promptOptions}
    </sl-select>`
  }

  private renderShowHideContent() {
    return html`<sl-tooltip
      content=${this.showContent ? 'Hide content' : 'Show content'}
    >
      <stencila-ui-icon-button
        name=${this.showContent ? 'eyeSlash' : 'eye'}
        @click=${(e: Event) => {
          // Stop the click behavior of the card header parent element
          e.stopImmediatePropagation()
          this.showContent = !this.showContent
        }}
      ></stencila-ui-icon-button>
    </sl-tooltip>`
  }

  private renderContent() {
    return html`<div
      class="max-w-prose mx-auto p-3"
      style="color: var(--default-text-colour);"
    >
      <slot name="content"></slot>
    </div>`
  }
}
