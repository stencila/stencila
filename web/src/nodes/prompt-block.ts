import { InstructionType } from '@stencila/types'
import { apply, css } from '@twind/core'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { patchValue } from '../clients/commands'
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
  @property({ attribute: 'instruction-type' })
  instructionType?: InstructionType

  @property()
  query?: string

  @property()
  target?: string

  /**
   * UI settings to use when rendering
   *
   * Instantiated in `connectedCallback` to avoid getting on each render.
   */
  private ui: NodeTypeUI

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
    this.requestUpdate()
  }

  /**
   * On a change to the `target` prompt property, send a patch to update it
   */
  private onPromptChanged(event: InputEvent) {
    let id = (event.target as HTMLInputElement).value

    if (id.trim().length === 0) {
      id = null
    } else {
      id = this.shortenPromptId(id)
    }

    this.dispatchEvent(patchValue('PromptBlock', this.id, 'target', id))
  }

  /**
   * Shorten a prompt id if possible
   *
   * Equivalent to the Rust `prompts::shorten` function.
   */
  private shortenPromptId(id: string): string {
    if (id.startsWith('stencila/')) {
      id = id.slice(9)
    }

    if (this.instructionType) {
      const prefix = `${this.instructionType.toLowerCase()}/`
      if (id.startsWith(prefix)) {
        id = id.slice(prefix.length)
      }
    }

    return id
  }

  /**
   * Expand a prompt id to a 'full' id
   *
   * Removes `?` suffix for inferred prompts.
   */
  private expandPromptId(id: string): string {
    if (id.endsWith('?')) {
      id = id.slice(0, -1)
    }

    const parts = id.split('/').length

    if (parts === 1) {
      return this.instructionType
        ? `stencila/${this.instructionType.toLowerCase()}/${id}`
        : `stencila/create/${id}`
    } else if (parts === 2) {
      return `stencila/${id}`
    } else {
      return id
    }
  }

  /**
   * On a change to the implied query, patch the query if it is null or
   * implied (ends in three spaces) and the target is null or inferred (ends with ?)
   */
  public onQueryImplied(query: string) {
    if (
      (!this.query || this.query.endsWith('   ')) &&
      (!this.target || this.target.endsWith('?'))
    ) {
      this.dispatchEvent(
        patchValue('PromptBlock', this.id, 'query', query + '   ')
      )
    }
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

    const inChat = this.isWithin('Chat')

    const { colour, textColour, borderColour } = this.ui

    const headerClasses = apply(
      'flex flex-row items-center gap-x-2',
      'w-full',
      `bg-[${colour}]`,
      `text-[${textColour}] text-xs leading-tight font-sans`,
      inChat ? '' : `px-3 py-1 border-t border-[${borderColour}]`
    )

    // Render as the property of a chat or instruction block
    if (this.isProperty()) {
      return html`
        <div>
          <div class=${headerClasses}>
            <label class=${inChat ? 'hidden' : ''}>Prompt </label>
            ${this.renderPromptSelect(borderColour, textColour)}
            ${this.renderShowHideContent()}
          </div>

          <stencila-ui-collapsible-animation
            class="${this.showContent ? 'opened' : ''}"
          >
            ${this.renderContent()}
          </stencila-ui-collapsible-animation>
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
          Prompt ${this.renderPromptSelect(borderColour, textColour)}
        </div>
      </div>

      <div slot="content" class="mx-auto">${this.renderContent()}</div>
    </stencila-ui-block-in-flow>`
  }

  private renderPromptSelect(borderColour: string, textColour: string) {
    // Filter prompts if necessary
    const prompts = this.instructionType
      ? data.prompts.filter(
          (prompt) => prompt.instructionTypes[0] === this.instructionType
        )
      : data.prompts

    // Render the prompt options
    const promptOption = (prompt: Prompt) => html`
      <sl-option value=${prompt.name} style="--sl-spacing-x-small: 0.25rem;">
        <span class="text-sm text-[${textColour}]">${prompt.name}</span>
        <span class="text-xs text-[${textColour}]/70 max-w-60 truncate">
          ${prompt.description}</span
        >
      </sl-option>
    `

    let options
    if (this.instructionType) {
      // Only show prompts for the instruction type
      options = prompts.map(promptOption)
    } else {
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
      options = Object.entries(types).map(([type, prompts], index) => {
        return html`
          ${index !== 0 ? html`<sl-divider class="my-1"></sl-divider>` : ''}
          <div
            class="flex flex-row items-center gap-2 px-2 py-1 text-[${textColour}]"
          >
            <stencila-ui-icon
              slot="prefix"
              class="text-base"
              name=${iconMaybe(type.toLowerCase()) ?? 'circle'}
            ></stencila-ui-icon>
            ${type}
          </div>
          ${prompts.map(promptOption)}
        `
      })
    }

    // Expand target prompt id so that is matches prompts
    let target = this.target ? this.expandPromptId(this.target) : null

    // If target is not in options, add one for it
    if (target) {
      const matched = prompts.find((prompt) => prompt.name == target)
      if (!matched) {
        // Use original, since not matched
        target = this.target

        options.unshift(
          promptOption({
            name: target,
            title: '',
            description: '',
            version: '',
            instructionTypes: [],
            nodeTypes: [],
            instructionPatterns: [],
          })
        )
      }
    }

    const style = css`
      &::part(combobox) {
        border-color: ${borderColour};
      }
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
      size="small"
      value=${target}
      @sl-change=${(e: InputEvent) => this.onPromptChanged(e)}
    >
      ${options}
    </sl-select>`
  }

  private renderShowHideContent() {
    return html`<sl-tooltip
      content=${this.showContent ? 'Hide content' : 'Show content'}
    >
      <stencila-ui-icon-button
        class="text-base"
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
    return html`
      <div class="bg-white/50 w-full max-h-[90vh] rounded overflow-y-auto mt-2">
        <div
          class="max-w-prose mx-auto p-3"
          style="color: var(--default-text-colour);"
        >
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
