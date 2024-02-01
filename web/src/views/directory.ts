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
   *
   * This server's root directory is what this view is intended
   * for and, at the time of writing, none of the directory
   * actions will succeed for any other directory.
   *
   * If not set as an attribute on the element (it normally shouldn't)
   * the id of the server's root directory will fetched in the constructor.
   */
  @property()
  doc?: DocumentId

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
  private directoryClient: DirectoryClient

  override async connectedCallback() {
    super.connectedCallback()

    if (this.doc === undefined) {
      this.doc = await DirectoryView.openFile('*')
    }

    this.objectClient = new ObjectClient(this.doc)
    this.objectClient.subscribe((_, { node }) => {
      this.directory = node as Directory
      this.requestUpdate()
    })

    this.directoryClient = new DirectoryClient(this.doc, this)
  }

  /**
   * Open a file on the server
   *
   * Returns the id of the document which can be used to open a view for the
   * document (by setting the `doc` attribute of the view element) e.g.
   *
   *   const view = document.createElement('stencila-live-view')
   *   view.setAttribute('doc', docId)
   *
   * @param path The path of the file
   */
  static async openFile(path: string): Promise<DocumentId> {
    const response = await fetch('/~open/' + path)
    if (response.status !== 200) {
      // TODO: Better error handling
      console.error(response)
    }
    const doc = await response.json()
    return doc.id
  }

  /**
   * Create a file
   *
   * @param parentPath The path of the parent directory
   * @param fileName The name of the file to create
   */
  createFile(parentPath: string, fileName: string) {
    this.directoryClient.sendAction('create-file', `${parentPath}/${fileName}`)
  }

  /**
   * Create a directory
   *
   * @param parentPath The path of the parent directory
   * @param directoryName The name of the directory to create
   */
  createDirectory(parentPath: string, directoryName: string) {
    this.directoryClient.sendAction(
      'create-directory',
      `${parentPath}/${directoryName}`
    )
  }

  /**
   * Delete a file or directory
   *
   * @param path The current path of the file or directory
   */
  delete(path: string) {
    this.directoryClient.sendAction('delete', path)
  }

  /**
   * Rename/move a file or directory
   *
   * @param oldPath The current path of the file or directory
   * @param newPath The new path
   */
  rename(oldPath: string, newPath: string) {
    this.directoryClient.sendAction('rename', oldPath, newPath)
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
