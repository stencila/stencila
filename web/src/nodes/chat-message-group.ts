import { html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { patchValue } from '../clients/commands'
import { withTwind } from '../twind'
import { type ModelData } from '../ui/nodes/chat/chat-group-model-tab'

import { ChatMessage } from './chat-message'
import { Entity } from './entity'
import { SoftwareApplication } from './software-application'

import '../ui/nodes/chat/chat-group-model-tab'

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
      // fetch the model info by getting attributes from the author slot of message
      // TODO: add model info to attributes of chat-group?
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
      <div class="mt-4">
        ${this.renderGroupHeader()}
        <div class="min-w-[45ch] max-w-[80ch] mx-auto mb-3">
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
          return html`
            <stencila-ui-chat-group-model-tab
              ?isSelected=${this.selectedMessage === i}
              .message=${this.messages[i]}
              .model=${m}
              .onSelect=${() => this.setSelected(i)}
            >
            </stencila-ui-chat-group-model-tab>
          `
        })}
      </div>
    `
  }
}
