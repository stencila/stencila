import { apply } from '@twind/core'
import { LitElement, PropertyValues, html } from 'lit'
import { customElement, state } from 'lit/decorators'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { withTwind } from '../../twind'

@customElement('stencila-image-drop-container')
@withTwind()
export class ImageDropContainer extends LitElement {
  private preventDefaults = (e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }

  private handleDragEnter = (e: DragEvent) => {
    this.preventDefaults(e)

    console.log('drag enter')
  }

  private handleDragLeave = (e: DragEvent) => {
    this.preventDefaults(e)

    console.log('drag leave')
  }

  private handleDrop = (e: DragEvent) => {
    this.preventDefaults(e)
    console.log('drop')
    this.dropEvent(e)
  }

  private fileInputRef: Ref<HTMLInputElement> = createRef()

  static acceptedFileTypes = ['image/png', 'image/jpeg', 'image/svg+xml']

  @state()
  files: File[] = []

  private dropEvent = (e: DragEvent) => {
    const dt = e.dataTransfer
    const files = dt.files

    ;[...files].forEach((file) => {
      if (ImageDropContainer.acceptedFileTypes.includes(file.type)) {
        this.files.push(file)
      }
    })
  }

  protected override firstUpdated(_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)

    const container = this.shadowRoot.querySelector('.drop-container')
    // apply all necesesary event listeners
    if (container) {
      container.addEventListener('dragover', this.preventDefaults)
      container.addEventListener('dragenter', this.handleDragEnter)
      container.addEventListener('dragleave', this.handleDragLeave)
      container.addEventListener('drop', this.handleDrop)
    }
  }

  override render() {
    const styles = apply([
      'h-full',
      'p-2',
      'border-2 border-dashed border-black/20 rounded-md',
    ])

    return html`
      <div class="drop-container ${styles}">
        <div>
          Drag and Drop Images, or
          <button
            class="underline hover:text-gray-500"
            @click=${() => this.fileInputRef.value?.click()}
          >
            click here
          </button>
          to browse
        </div>
        <input
          ${ref(this.fileInputRef)}
          type="file"
          hidden
          multiple
          @change=${(e: Event) => {
            // @ts-expect-error "EventTarget does not have `files` property"
            this.files.push(...e.target.files)
          }}
        />
      </div>
    `
  }
}
