import { ExecutionStatus } from '@stencila/types'
import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { ChatMessage } from '../../../nodes/chat-message'
import { withTwind } from '../../../twind'
import { iconMaybe } from '../../icons/icon'

export type ModelData = {
  id: string
  name: string
  version: string
}

@customElement('stencila-ui-chat-group-model-tab')
@withTwind()
export class UIChatModelTab extends LitElement {
  @property({ type: Array })
  message: ChatMessage

  @property({ type: Object })
  model: ModelData

  @property({ type: Boolean })
  isSelected: boolean = false

  @property({ type: Function })
  onSelect: () => void

  @state()
  messageStatus: ExecutionStatus

  private messageObserver: MutationObserver

  private getTextColour() {
    switch (this.messageStatus) {
      case 'Warnings':
        return 'yellow-800'
      case 'Exceptions':
      case 'Errors':
        return 'red-800'
      case 'Succeeded':
        return 'black'
      default:
        return 'gray-400'
    }
  }

  override connectedCallback(): void {
    super.connectedCallback()
    this.messageObserver = new MutationObserver(
      (mutations: MutationRecord[]) => {
        mutations.forEach((m) => {
          if (
            m.type === 'attributes' &&
            m.attributeName === 'execution-status'
          ) {
            this.messageStatus = (m.target as ChatMessage).executionStatus
          }
        })
      }
    )
    this.messageObserver.observe(this.message, { attributes: true })
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    this.messageObserver.disconnect()
  }

  protected override render() {
    const [provider, _name] = this.model.id.trim().split('/') ?? []

    let icon = iconMaybe(this.model.id.toLowerCase())

    // Fallback to using name
    if (!icon) {
      icon = iconMaybe(this.model.name.toLowerCase())
    }

    if (!icon) {
      icon = iconMaybe(provider.toLowerCase())
    }

    // Fallback to using prompt icon if appropriate
    if (!icon) {
      icon = 'chatSquareText'
    }

    const providerTitle =
      provider === 'openai'
        ? 'OpenAi'
        : `${provider.charAt(0).toUpperCase()}${provider.slice(1)}`

    return html`
      <button
        class="flex items-center font-sans p-2 ${this.isSelected
          ? 'text-brand-blue font-semibold'
          : `text-${this.getTextColour()}`}"
        @click=${this.onSelect}
      >
        <stencila-ui-icon name=${icon} class="text-2xl"></stencila-ui-icon>
        <div class="flex flex-col justify-center ml-2">
          <span class="text-xs text-left leading-5"
            >${providerTitle} ${this.model.name}</span
          >
          <span class="text-2xs text-left inline-block leading-none">
            ${this.model.version ?? '1.0.1'}
          </span>
        </div>
      </button>
    `
  }
}
