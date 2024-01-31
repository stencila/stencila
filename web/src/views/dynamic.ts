import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { DomClient } from '../clients/dom'
import { NodesClient } from '../clients/nodes'
import '../nodes'
import type { DocumentId, DocumentAccess } from '../types'

import { ThemedView } from './themed'

/**
 * Dynamic view of a document
 *
 * A view which, in addition to providing live updates of a document,
 * allows for the user to change input values (e.g. the `value` of a `Parameter` node)
 */
@customElement('stencila-dynamic-view')
export class DynamicView extends ThemedView {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  /**
   * The access level of the view
   *
   * This property is passed through to the `NodesClient` but may also
   * be inspected by descendent WebComponents to determine their behavior.
   *
   * This should not be `edit`, `write` or `admin` since this view
   * does not provide the means to modify those.
   */
  @property()
  access: DocumentAccess = 'code'

  /**
   * A read-only client which updates the document's DOM when the
   * document changes on the server
   */
  // @ts-expect-error "dom client is set, not read"
  private domClient: DomClient

  /**
   * A write-only client which sends node patches to the document
   * on the server
   */
  // @ts-expect-error "nodes client is set, not read"
  private nodesClient: NodesClient

  /**
   * Override so that clients are instantiated _after_ this
   * element has a document `[data-root]` element in its `renderRoot`.
   */
  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    this.domClient = new DomClient(
      this.doc,
      this.renderRoot.firstElementChild as HTMLElement
    )

    this.nodesClient = new NodesClient(
      this.doc,
      this.access,
      this.renderRoot as HTMLElement,
      'dynamic'
    )
  }

  override render() {
    return html`<stencila-article root></stencila-article>`
  }
}
