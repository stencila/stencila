import { File } from '@stencila/types'
import { html } from 'lit'
import { customElement, state } from 'lit/decorators.js'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { runChat } from '../../clients/commands'
import { withTwind } from '../../twind'
import { debounce } from '../../utilities/debounce'
import { fileToStencilaFile } from '../inputs/file-input'

import { UIBaseClass } from './mixins/ui-base-class'

@customElement('stencila-ui-chat-message-inputs')
@withTwind()
export class UIChatMessageInputs extends UIBaseClass {
  /**
   * Whether there is any text input to send
   *
   * Boolean state to update re-render only when necessary (not on
   * every keypress that changes text, but only when changes from
   * empty to not empty or vice verse).
   */
  @state()
  private hasText: boolean = false

  /**
   * Files attached to the message
   */
  @state()
  private files: File[] = []

  /**
   * Reference to <textarea> used to focus and get value
   */
  private textRef: Ref<HTMLTextAreaElement> = createRef()

  /**
   * Debounced event emitter for text input
   *
   * Used to bubble up text input events to chat component but
   * not on every keypress.
   */
  private debouncedTextInput: (value: string) => void

  /**
   * On <textarea> input adjust the height if necessary and
   * update whether this has non-empty text
   */
  private onTextInput(event: InputEvent) {
    const textarea = event.target as HTMLTextAreaElement

    textarea.style.height = 'auto'
    textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`

    this.hasText = textarea.value.trim().length > 0

    this.debouncedTextInput(textarea.value)
  }

  /**
   * On <textarea> keydown send if enter key, but not Shift+Enter
   */
  private onTextKeyDown(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault()
  
      const textarea = event.target as HTMLTextAreaElement
      if (textarea.value.trim().length > 0) {
        this.onSend()
      }
    }
  }

  /**
   * On paste into <textarea> if any files then add to
   * file list
   */
  private async onTextPaste(event: ClipboardEvent) {
    const clipboardData = event.clipboardData
    if (clipboardData.types.includes('Files')) {
      event.preventDefault()

      const files = await Promise.all(
        Array.from(clipboardData.items)
          .filter((item) => item.kind === 'file')
          .map((item) => fileToStencilaFile(item.getAsFile()))
      )

      this.files = [...this.files, ...files]
    }
  }

  /**
   * On file input add files to file list
   */
  private onFileInput(event: CustomEvent) {
    this.files = [...this.files, ...event.detail]
  }

  /**
   * On file tag remove, remove the file from the file list
   */
  private onFileRemove(name: string) {
    this.files = this.files.filter((file) => file.name !== name)
  }

  /**
   * Send the message
   */
  onSend() {
    const text = this.textRef.value.value
    const files = this.files

    this.dispatchEvent(runChat(this.nodeId, text, files))

    // Clear the text area and files list
    const textarea = this.textRef.value
    textarea.value = ''
    textarea.style.height = 'auto'
    textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`
    this.hasText = false
    this.files = []
  }

  override connectedCallback() {
    super.connectedCallback()

    this.debouncedTextInput = debounce(
      (value: string) =>
        this.dispatchEvent(
          new CustomEvent('stencila-message-input', { detail: value })
        ),
      300
    )
  }

  override firstUpdated() {
    this.textRef.value.focus()
  }

  override render() {
    const { borderColour, textColour } = this.ui

    const hasInputs = this.hasText || this.files.length > 0

    const files = this.files.map(
      (file) =>
        html`<sl-tag
          size="small"
          removable
          @sl-remove=${() => this.onFileRemove(file.name)}
          ><span class="font-sans text-[${textColour}]"
            >${file.name}</span
          ></sl-tag
        >`
    )

    return html`
      <div class="bg-white rounded border border-[${borderColour}] p-2">
        <textarea
          class="w-full resize-none overflow-hidden outline-none px-1 py-1"
          placeholder=""
          rows=${1}
          @input=${this.onTextInput}
          @keydown=${this.onTextKeyDown}
          @paste=${this.onTextPaste}
          ${ref(this.textRef)}
        ></textarea>

        <div class="flex flex-row items-start justify-between">
          <div>
            <sl-tooltip content="Add file">
              <stencila-ui-file-input
                class="text-xl text-[${textColour}]"
                multiple
                @stencila-files=${this.onFileInput}
              ></stencila-ui-file-input>
            </sl-tooltip>

            ${files}
          </div>

          <sl-tooltip content=${hasInputs ? 'Send message' : 'Message is empty'}
            ><stencila-ui-icon-button
              class="text-xl ${hasInputs
                ? `text-[${textColour}]`
                : 'text-grey-500'}"
              name=${hasInputs ? 'arrowUpCircleFill' : 'arrowUpCircle'}
              ?disabled=${!hasInputs}
              @click=${this.onSend}
            ></stencila-ui-icon-button
          ></sl-tooltip>
        </div>
      </div>
    `
  }
}
