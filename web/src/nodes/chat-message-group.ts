import { css as twindCss } from '@twind/core'
import { html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { patchValue } from '../clients/commands'
import { withTwind } from '../twind'
import { iconMaybe } from '../ui/icons/icon'

import { ChatMessage } from './chat-message'
import { Entity } from './entity'
import { SoftwareApplication } from './software-application'

type ModelData = {
  id: string
  name: string
  version: string
}

/**
 * Web component representing a Stencila `ChatMessageGroup`
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/chat-message-group.md
 */
@customElement('stencila-chat-message-group')
@withTwind()
export class ChatMessageGroup extends Entity {
  /**
   * Array of chat messages in the group
   */
  @state()
  messages: ChatMessage[] = []

  /**
   * Index of tahe currently selected message in the `messages` array
   */
  @state()
  selectedMessage: number = 0

  /**
   * Array of models used for the messages in the group
   */
  @state()
  models: ModelData[] = []

  private handleMessageSlotChange(e: Event) {
    const slotEl = e.target as HTMLSlotElement

    const messageElements = Array.from(
      slotEl.assignedElements()[0].children
    ) as ChatMessage[]

    if (messageElements.length > 0) {
      const models: ModelData[] = []
      messageElements.forEach((msg) => {
        const softwareAppElement =
          msg.querySelector('[slot="author"]').firstElementChild

        if (softwareAppElement instanceof SoftwareApplication) {
          const model = {
            id: softwareAppElement.$id,
            name: softwareAppElement.name,
            version: softwareAppElement.version,
          }
          models.push(model)
        }
      })
      this.messages = messageElements
      this.models = models
      this.setSelected(this.selectedMessage)
    }
  }

  private setSelected(index: number) {
    if (this.messages.length > 0) {
      // return if selected index is already selected
      if (index === this.selectedMessage) {
        return
      }

      // update properties and dispatch event
      this.messages[this.selectedMessage].isSelected = false
      this.messages[index].isSelected = true
      this.selectedMessage = index
      this.dispatchEvent(
        patchValue(
          'ChatMessageGroup',
          this.id,
          ['messages', index, 'isSelected'],
          true
        )
      )
    }
  }

  override render() {
    return html`
      <div>
        ${this.renderGroupHeader()}
        <div class="min-w-[45ch] max-w-prose mx-auto mb-3">
          <slot
            name="messages"
            @slotchange=${this.handleMessageSlotChange}
          ></slot>
        </div>
      </div>
    `
  }

  renderGroupHeader() {
    return html`
      <div class="flex flex-row justify-center gap-4 w-full">
        ${this.models.map((m, i) => {
          return html`${this.renderModelTab(m, i)}`
        })}
      </div>
    `
  }

  renderModelTab(model: ModelData, index: number) {
    let icon = iconMaybe(model.id.toLowerCase())

    // Fallback to using name
    if (!icon) {
      icon = iconMaybe(model.name.toLowerCase())
    }

    if (!icon) {
      const [provider] = model.id?.trim().split('/') ?? []
      icon = iconMaybe(provider.toLowerCase())
    }

    // Fallback to using prompt icon if appropriate
    if (!icon) {
      icon = 'chatSquareText'
    }

    const style = twindCss`
      box-shadow: 0px 0px 4px 0px rgba(0, 0, 0, 0.25);
    `

    const isCurrent = this.selectedMessage === index

    return html`
      <button
        class="flex items-center rounded-sm p-2 ${isCurrent
          ? 'text-brand-blue'
          : 'text-gray-500'} ${style}"
        @click=${() => this.setSelected(index)}
      >
        <stencila-ui-icon name=${icon} class="text-2xl"></stencila-ui-icon>
        <span class="text-sm ml-1">${model.version}</span>
      </button>
    `
  }
}
