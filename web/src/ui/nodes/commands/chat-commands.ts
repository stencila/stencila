import { InstructionType } from '@stencila/types'
import { css } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import {
  insertClone,
  insertInstruction,
  patchChatFocus,
} from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { closestGlobally } from '../../../utilities/closestGlobally'
import { IconName } from '../../icons/icon'
import { UIBaseClass } from '../mixins/ui-base-class'

/**
 * Commands available on a node when it is within a chat
 */
@customElement('stencila-ui-node-chat-commands')
@withTwind()
export class UINodeChatCommands extends UIBaseClass {
  /**
   * The instruction type associated with the chat
   *
   * Used to determine which types of instructions
   * to include in menu.
   */
  @property({ attribute: 'instruction-type' })
  instructionType?: InstructionType

  /**
   * Insert a clone of the node into the active document
   */
  private onInsertClone() {
    this.dispatchEvent(insertClone([this.nodeId]))
  }

  /**
   * Insert an instruction with a clone of the node into
   * the active document
   */
  private onInsertInstruction(type: InstructionType) {
    this.dispatchEvent(insertInstruction([this.nodeId], type, 'Auto'))
  }

  /**
   * Insert an edit instruction for a clone of the node
   * into the active document
   */
  private onInsertEdit() {
    this.onInsertInstruction('Edit')
  }

  /**
   * Insert a fix instruction for a clone of the node
   * into the active document
   */
  private onInsertFix() {
    this.onInsertInstruction('Fix')
  }

  /**
   * Make the current node the focus of the chat
   */
  private onFocus() {
    const chatId = closestGlobally(this, 'stencila-chat')?.id
    this.dispatchEvent(patchChatFocus(chatId, this.nodeId))
  }

  override render() {
    // Do not show these commands for nodes not in a chat, or within a chat
    // but inside a suggestion block
    if (
      !closestGlobally(this, 'stencila-chat') ||
      closestGlobally(this, 'stencila-suggestion-block')
    ) {
      return ''
    }

    const { borderColour, textColour, title } = this.ui

    const menuItemClass = css`
      &::part(checked-icon),
      &::part(submenu-icon) {
        display: none;
      }
    `

    const name = title.toLowerCase()

    const useCommands: [IconName, string, string, () => void][] = [
      [
        'boxArrowInLeft',
        'Copy',
        `Insert this ${name} into doc.`,
        this.onInsertClone,
      ],
    ]

    if (['Paragraph'].includes(this.type)) {
      useCommands.push([
        'circle',
        'Auto Edit',
        `Insert an edit command for this ${name} into doc.`,
        this.onInsertEdit,
      ])
    }

    if (['CodeChunk', 'MathBlock'].includes(this.type)) {
      useCommands.push([
        'circle',
        'Auto Fix',
        `Insert a fix command for this ${name} into doc.`,
        this.onInsertFix,
      ])
    }

    const menuItems = useCommands.map(
      ([icon, label, help, handler]) =>
        html`<sl-menu-item class=${menuItemClass} @click=${handler}>
          <div class="px-2 text-[${textColour}]">
            <div class="flex flex-row gap-2">
              <stencila-ui-icon name=${icon as IconName}></stencila-ui-icon>
              <span class="text-xs">${label}</span>
            </div>
            <div class="mt-1 text-[0.65rem] opacity-70 whitespace-normal">
              ${help}
            </div>
          </div>
        </sl-menu-item>`
    )

    const [icon, _label, help, handler] = useCommands[0]
    const useButton = html`<sl-tooltip content=${help}>
      <stencila-ui-icon-button
        class="text-xl ml-1"
        name=${icon}
        @click=${handler}
      ></stencila-ui-icon-button>
    </sl-tooltip>`

    const targetButton = html`<sl-tooltip
      content="Focus the chat on this ${name}"
    >
      <stencila-ui-icon-button
        class="text-xl ml-2"
        name="crosshair"
        @click=${this.onFocus}
      ></stencila-ui-icon-button>
    </sl-tooltip>`

    return html`
      <div class="flex flex-row items-center text-${textColour}">
        <sl-dropdown>
          <stencila-ui-icon-button
            class="text-xs"
            name="chevronDown"
            slot="trigger"
          ></stencila-ui-icon-button>

          <sl-menu class="rounded border border-[${borderColour}] w-72 z-50">
            ${menuItems}
          </sl-menu>
        </sl-dropdown>

        ${useButton} ${targetButton}
      </div>
    `
  }
}
