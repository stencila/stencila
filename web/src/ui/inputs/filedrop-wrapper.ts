import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, query, state } from 'lit/decorators'

import { withTwind } from '../../twind'

/**
 * A wrapper component to allow the drag and drop of files
 */
@customElement('stencila-ui-filedrop-wrapper')
@withTwind()
export class UIFileDropWrapper extends LitElement {
  @query('div.drop-container')
  dropContainer: HTMLDivElement

  /**
   * Static array of accepted file types
   */
  static acceptedFileTypes = ['image/png', 'image/jpeg', 'image/svg+xml']

  @property({ type: Function })
  dropEvent: (e: DragEvent) => void

  /**
   * The selected files
   *
   * TODO: Note that we do not get the the full path of the file
   * and so it is necessary to use the JS FileReader to get
   * the data
   */
  @state()
  files: File[] = []

  /**
   * The dragover state of the wrapper.
   */
  @state()
  dragover: boolean = false

  private preventDefaults = (e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }

  private handleDragEnter = (e: DragEvent) => {
    this.preventDefaults(e)
    if (!this.dragover) {
      this.dragover = true
    }
  }

  // TODO: Handle case where leave event fires when dragging over child elements
  private handleDragLeave = (e: DragEvent) => {
    this.preventDefaults(e)
    const target = e.relatedTarget as Node
    if (!this.contains(target) && !this.dropContainer.contains(target)) {
      this.dragover = false
    }
  }

  private handleDrop = (e: DragEvent) => {
    this.preventDefaults(e)
    this.dragover = false
    this.dropEvent(e)
  }

  override connectedCallback(): void {
    super.connectedCallback()
    this.addEventListener('dragenter', this.handleDragEnter)
    this.addEventListener('dragover', this.handleDragEnter)
    this.addEventListener('dragleave', this.handleDragLeave)
    this.addEventListener('drop', this.handleDrop)
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    this.removeEventListener('dragenter', this.handleDragEnter)
    this.removeEventListener('dragover', this.preventDefaults)
    this.removeEventListener('dragleave', this.handleDragLeave)
    this.removeEventListener('drop', this.handleDrop)
  }

  override render() {
    const styles = apply([
      'h-full w-full',
      'border-2 border-dashed border-transparent',
      this.dragover ? ' border-black/50' : '',
    ])

    return html`
      <div class="drop-container ${styles}">
        <slot></slot>
      </div>
    `
  }
}
