import { LitElement, html } from 'lit'
import { state, customElement } from 'lit/decorators'
import { ref, Ref, createRef } from 'lit/directives/ref'

import '../../../inputs/imagedrop'

import { withTwind } from '../../../../twind'

@customElement('stencila-ui-instruction-editor')
@withTwind()
export class InstructionEditor extends LitElement {
  private fileInputRef: Ref<HTMLInputElement> = createRef()

  static acceptedFileTypes = ['image/png', 'image/jpeg', 'image/svg+xml']

  @state()
  private files: File[]

  private dropEvent = (e: DragEvent) => {
    const dt = e.dataTransfer
    const files = dt.files

    ;[...files].forEach((file) => {
      if (InstructionEditor.acceptedFileTypes.includes(file.type)) {
        this.files.push
      }
    })

    console.log('drop')
  }

  protected override render() {
    return html`
      <stencila-image-drop-container .dropEvent=${this.dropEvent}>
        <div class="flex flex-row">
          <input class="grow" type="text" />
          <input
            ${ref(this.fileInputRef)}
            hidden
            type="file"
            multiple
            accept="image/*"
          />
          <button
            @click=${() => {
              this.fileInputRef.value.click()
            }}
          >
            <sl-icon name="images" library="default"></sl-icon>
          </button>
        </div>
      </stencila-image-drop-container>
    `
  }
}
