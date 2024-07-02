import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

@customElement('stencila-image-drop-container')
@withTwind()
export class ImageDropContainer extends LitElement {
  @property()
  dropEvent: (e: DragEvent) => void

  /**
   *  List of accepted file mime types
   */
  static acceptedFileTypes = ['image/png', 'image/jpeg', 'image/svg+xml']

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

    this.dropEvent && this.dropEvent(e)
  }

  override connectedCallback() {
    super.connectedCallback()

    const container = this.shadowRoot.querySelector('div.drop-container')

    // apply all necesesary event listeners
    container.addEventListener('dragover', this.preventDefaults)
    container.addEventListener('dragenter', this.handleDragEnter)
    container.addEventListener('dragleave', this.handleDragLeave)
    container.addEventListener('drop', this.handleDrop)
  }

  protected override render() {
    const styles = apply(['border border-black/20 rounded-md', 'p-2'])

    return html`
      <div class="drop-container ${styles}">
        <slot></slot>
      </div>
    `
  }
}
