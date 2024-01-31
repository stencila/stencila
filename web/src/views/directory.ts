import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { DirectoryClient } from '../clients/directory'
import { DomClient } from '../clients/dom'
import '../nodes/directory'
import '../nodes/file'
import type { DocumentId } from '../types'

/**
 * Tree view of a directory
 *
 * Updates as the directory changes on the files system (i.e. when
 * files or subdirectories are created, deleted or renamed).
 *
 * Uses the `DirectoryClient` to provide buttons to create, rename,
 * and delete files and subdirectories.
 */
@customElement('stencila-directory-view')
export class DirectoryView extends LitElement {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  /**
   * A read-only client which updates the DOM representation of
   * the directory when it changes
   */
  // @ts-expect-error "property is written to, not read"
  private domClient: DomClient

  /**
   * A write-only client which captures custom events for creating,
   * renaming and deleting files and subdirectories and sends them
   * to the server for handling
   */
  // @ts-expect-error "property is written to, not read"
  private directoryClient: DirectoryClient

  /**
   * Override so that the `DomClient` is instantiated _after_ this
   * element has a document `[root]` element in its `renderRoot`.
   */
  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    this.domClient = new DomClient(
      this.doc,
      this.renderRoot.querySelector('[root]')
    )

    this.directoryClient = new DirectoryClient(
      this.doc,
      this.renderRoot.querySelector('[root]')
    )
  }

  override render() {
    return html`<sl-tree class="tree-with-lines tree-with-icons">
      <stencila-directory root></stencila-directory>
    </sl-tree>`
  }
}
