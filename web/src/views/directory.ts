import type { File, Directory } from '@stencila/types'
import { LitElement, TemplateResult, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { DirectoryClient } from '../clients/directory'
import { ObjectClient } from '../clients/object'
import type { DocumentId } from '../types'

/**
 * Tree view of a directory
 *
 * Uses the `ObjectClient` to update the view and the directory changes on the
 * file system (i.e. when files or subdirectories are created, deleted or renamed).
 *
 * Uses the `DirectoryClient` to provide buttons to create, rename,
 * and delete files and subdirectories.
 */
@customElement('stencila-directory-view')
export class DirectoryView extends LitElement {
  /**
   * The id of the document for the directory
   */
  @property()
  doc: DocumentId

  /**
   * The directory as a JavaScript object
   *
   * Synced from the server by the `objectClient`
   */
  private directory: Directory

  /**
   * A read-only client which updates the DOM representation of
   * the directory when it changes
   */
  private objectClient: ObjectClient

  /**
   * A write-only client which captures custom events for creating,
   * renaming and deleting files and subdirectories and sends them
   * to the server for application
   */
  // @ts-expect-error "is declared but its value is never read"
  private directoryClient: DirectoryClient

  override connectedCallback() {
    super.connectedCallback()

    this.objectClient = new ObjectClient(this.doc)
    this.objectClient.subscribe((_, { node }) => {
      this.directory = node as Directory
      this.requestUpdate()
    })

    this.directoryClient = new DirectoryClient(this.doc, this)
  }

  override render() {
    return html`<sl-tree class="tree-with-lines tree-with-icons">
      ${this.directory ? this.renderDirectory(this.directory) : ''}
    </sl-tree>`
  }

  private renderDirectory(directory: Directory): TemplateResult {
    return html`<sl-tree-item>
      <sl-icon name="folder"></sl-icon> ${directory.name}
      ${directory.parts.map((part: Directory | File) =>
        part.type === 'Directory'
          ? this.renderDirectory(part as Directory)
          : this.renderFile(part)
      )}
    </sl-tree-item>`
  }

  private renderFile(file: File) {
    return html`<sl-tree-item
      ><sl-icon name="file"></sl-icon> ${file.name}</sl-tree-item
    >`
  }
}
