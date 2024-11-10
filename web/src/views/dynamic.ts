import { NodeType } from '@stencila/types'
import { customElement, property } from 'lit/decorators.js'

import { CommandsClient } from '../clients/commands'
import { DomClient } from '../clients/dom'
import type { DocumentId, DocumentAccess } from '../types'

import { DocumentView } from './document'

import '../nodes'
import '../shoelace'
import '../ui/document/menu'

/**
 * Dynamic view of a document
 *
 * A view which, in addition to providing live updates of a document,
 * allows for the user to change input values (e.g. the `value` of a `Parameter` node)
 */
@customElement('stencila-dynamic-view')
export class DynamicView extends DocumentView {
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
   * The type of the root node of the document e.g. Article, Prompt
   */
  @property()
  type: NodeType = 'Article'

  /**
   * A read-only client which updates the document's DOM when the
   * document changes on the server
   */
  // @ts-expect-error "dom client is set, not read"
  private domClient: DomClient

  /**
   * A write-only client which sends command to the document
   * on the server
   */
  // @ts-expect-error "nodes client is set, not read"
  private nodesClient: CommandsClient

  /**
   * Override to pass the render root to the clients
   */
  protected override createRenderRoot(): this {
    const renderRoot = super.createRenderRoot()

    this.domClient = new DomClient(this.doc, renderRoot as HTMLElement)

    this.nodesClient = new CommandsClient(
      this.doc,
      this.access,
      renderRoot as HTMLElement
    )

    return renderRoot
  }
}
