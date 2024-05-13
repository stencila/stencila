import { provide } from '@lit/context'
import { CSSResultGroup, LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { CommandsClient } from '../clients/commands'
import { DomClient } from '../clients/dom'
import {
  DocViewContext,
  documentViewContext,
} from '../contexts/article-context'
import type { DocumentId, DocumentAccess } from '../types'

import '../nodes'
import '../shoelace'

import { outputCSS } from './styles/global-styles'

/**
 * Dynamic view of a document
 *
 * A view which, in addition to providing live updates of a document,
 * allows for the user to change input values (e.g. the `value` of a `Parameter` node)
 */
@customElement('stencila-dynamic-view')
export class DynamicView extends LitElement {
  @provide({ context: documentViewContext })
  @state()
  protected context: DocViewContext = {
    showAllToggleChips: false,
  }

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
   * A write-only client which sends command to the document
   * on the server
   */
  // @ts-expect-error "nodes client is set, not read"
  private nodesClient: CommandsClient

  // Add outputCSS to view
  static override styles?: CSSResultGroup = [outputCSS]

  /**
   * Override so that this component has a Light DOM so that
   * theme styles apply to it.
   */
  protected override createRenderRoot() {
    return this
  }

  /**
   * Override so that clients are instantiated _after_ this
   * element has a document `[data-root]` element in its `renderRoot`.
   */
  override update(changedProperties: Map<string, string | boolean>): void {
    super.update(changedProperties)

    if (changedProperties.has('doc')) {
      this.domClient = new DomClient(
        this.doc,
        this.renderRoot.firstElementChild as HTMLElement
      )

      this.nodesClient = new CommandsClient(
        this.doc,
        this.access,
        this.renderRoot as HTMLElement
      )
    }
  }

  override render() {
    return html`
      <div>THISDFASDFASDFASDF</div>
      <stencila-article root></stencila-article>
    `
  }
}
