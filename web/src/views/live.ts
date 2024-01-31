import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { DomClient } from '../clients/dom'
import '../nodes'
import type { DocumentId } from '../types'

import { ThemedView } from './themed'

/**
 * Live view of a document
 *
 * A view which provides live updates of a document's DOM as it changes
 * on the server.
 */
@customElement('stencila-live-view')
export class LiveView extends ThemedView {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  /**
   * A read-only client which will update the document's DOM when the
   * document changes
   */
  // @ts-expect-error "dom client is set, not read"
  private domClient: DomClient

  /**
   * Override so that the `DomClient` is instantiated _after_ this
   * element has a document `[data-root]` element in its `renderRoot`.
   */
  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    this.domClient = new DomClient(
      this.doc,
      this.renderRoot.firstElementChild as HTMLElement
    )
  }

  override render() {
    return html`<stencila-article root></stencila-article>`
  }
}
