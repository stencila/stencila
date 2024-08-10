import { apply } from '@twind/core'
import { LitElement, PropertyValues, html } from 'lit'
import { customElement, state } from 'lit/decorators'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { withTwind } from '../../twind'

@customElement('stencila-ui-image-upload')
@withTwind()
export class UIImageUpload extends LitElement {
  static acceptedFileTypes = ['image/png', 'image/jpeg', 'image/svg+xml']

  /**
   * The selected files
   *
   * TODO: Note that we do not get the the full path of the file
   * and so it is necessary to use the JS FileReader to get
   * the data
   */
  @state()
  files: File[] = []

  private fileInputRef: Ref<HTMLInputElement> = createRef()

  private preventDefaults = (e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }

  private handleDragEnter = (e: DragEvent) => {
    this.preventDefaults(e)
  }

  private handleDragLeave = (e: DragEvent) => {
    this.preventDefaults(e)
  }

  private handleDrop = (e: DragEvent) => {
    this.preventDefaults(e)
    this.dropEvent(e)
  }

  private dropEvent = (e: DragEvent) => {
    const dt = e.dataTransfer
    const files = dt.files

    ;[...files].forEach((file) => {
      if (UIImageUpload.acceptedFileTypes.includes(file.type)) {
        this.files.push(file)
      }
    })
  }

  protected override firstUpdated(_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)

    const container = this.shadowRoot.querySelector('.drop-container')
    if (container) {
      container.addEventListener('dragover', this.preventDefaults)
      container.addEventListener('dragenter', this.handleDragEnter)
      container.addEventListener('dragleave', this.handleDragLeave)
      container.addEventListener('drop', this.handleDrop)
    }
  }

  override render() {
    const styles = apply(['h-full', 'p-3', 'font-sans'])

    console.log(this.files)

    return html`
      <div class="drop-container ${styles}">
        <div class="flex items-center">
          <stencila-ui-icon class="text-base mr-2" name="image"></stencila-ui-icon>
          <button
            class="hover:text-gray-800"
            @click=${() => this.fileInputRef.value?.click()}
          >
            Drag and drop, or click to add an image
          </button>
        </div>

        <input
          ${ref(this.fileInputRef)}
          type="file"
          accept=${UIImageUpload.acceptedFileTypes.join(', ')}
          hidden
          multiple
          @change=${(e: Event) => {
            // Must use spread operators and assignment here so that
            // the files `@state` update is triggered to perform a rerender.
            // @ts-expect-error "EventTarget does not have `files` property"
            this.files = [...this.files, ...e.target.files]
          }}
        />

        <div class="mt-1 font-mono text-xs"></div>
          ${this.files.map((file) => file.name).join(', ')}
        </div>
      </div>
    `
  }
}
