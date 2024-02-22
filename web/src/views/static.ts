import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { ExportClient } from '../clients/export'
import '../nodes'
import type { DocumentId } from '../types'

import { ThemedView as ThemedView } from './themed'

/**
 * Static view of a document
 *
 * A static view of a document which does not update as the document changes,
 * nor allows changes to it.
 */
@customElement('stencila-static-view')
export class StaticView extends ThemedView {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  /**
   * Whether the document's HTML should be fetched
   *
   * Used when the view is within an app to get the latest
   * content of the document.
   */
  @property({ type: Boolean })
  fetch: boolean

  /**
   * Fetch and set the current doc's html
   */
  private fetchHTML = () => {
    new ExportClient(this.doc, 'dom').fetch().then((html) => {
      this.shadowRoot.innerHTML = html
    })
  }

  /**
   * Override to fetch document's HTML if necessary
   */
  override connectedCallback() {
    super.connectedCallback()

    if (this.fetch) {
      this.fetchHTML()
    }
  }

  /**
   * Override to re fetch the html if the `doc` property has updated
   */
  override update(changedProperties: Map<string, string | boolean>): void {
    super.update(changedProperties)
    if (changedProperties.has('doc') && this.fetch) {
      this.fetchHTML()
    }
  }

  override render() {
    if (this.fetch) {
      // Render a spinner which will spin until fetched content replaces it
      return html`<p>TODO: Spinner</p>`
    } else {
      // Render a slot for server provided HTML to go into
      return html`<slot></slot>`
    }
  }
}
