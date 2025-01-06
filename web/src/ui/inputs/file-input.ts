import { File as StencilaFile } from '@stencila/types'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { withTwind } from '../../twind'

import '../buttons/icon'

/**
 * A file input button
 */
@customElement('stencila-ui-file-input')
@withTwind()
export class UIFileInput extends LitElement {
  /**
   * Whether multiple files can be input
   */
  @property({ type: Boolean })
  multiple: boolean = false

  /**
   * Whether the file input is disabled
   */
  @property({ type: Boolean })
  disabled: boolean = false

  /**
   * Reference to the <input> to forward a click
   * on the button to the <input>
   */
  private inputRef: Ref<HTMLInputElement> = createRef()

  /**
   * Handle click on button by forwarding event to <input>
   */
  onButtonClick() {
    this.inputRef.value?.click()
  }

  /**
   * Handle change on <input> by forwarding event to input handler
   */
  async onInputChange(event: InputEvent) {
    const input = event.target as HTMLInputElement

    const files = await Promise.all(
      Array.from(input.files).map(fileToStencilaFile)
    )

    this.dispatchEvent(new CustomEvent('stencila-files', { detail: files }))
  }

  protected override render() {
    return html`
      <stencila-ui-icon-button
        name="paperclip"
        ?disabled=${this.disabled}
        @click=${this.onButtonClick}
      ></stencila-ui-icon-button>
      <input
        type="file"
        hidden
        ?multiple=${this.multiple}
        ?disabled=${this.disabled}
        @change=${this.onInputChange}
        ${ref(this.inputRef)}
      />
    `
  }
}

export async function fileToStencilaFile(file: File): Promise<StencilaFile> {
  const name = file.name
  const mediaType = file.type
  const size = file.size

  let content: string | undefined
  let transferEncoding: string | undefined
  if (size > 0) {
    if (
      mediaType.startsWith('application/json') ||
      mediaType.startsWith('text')
    ) {
      content = await readFileAsText(file)
    } else {
      content = await readFileAsBase64(file)
      transferEncoding = 'base64'
    }
  }

  return new StencilaFile(name, '', {
    mediaType,
    transferEncoding,
    size,
    content,
  })
}

function readFileAsBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      if (reader.result) {
        resolve(reader.result.toString().split(',')[1])
      } else {
        reject(new Error('FileReader result is null'))
      }
    }
    reader.onerror = () => reject(reader.error)
    reader.readAsDataURL(file) // Read as data URL and extract Base64
  })
}

function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => {
      if (reader.result) {
        resolve(reader.result.toString())
      } else {
        reject(new Error('FileReader result is null'))
      }
    }
    reader.onerror = () => reject(reader.error)
    reader.readAsText(file) // Read as plain text
  })
}
