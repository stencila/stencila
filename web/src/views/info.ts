import '@shoelace-style/shoelace/dist/components/icon/icon'
import '@shoelace-style/shoelace/dist/components/tree/tree'
import '@shoelace-style/shoelace/dist/components/tree-item/tree-item'
import { consume } from '@lit/context'
import { css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { ref, Ref, createRef } from 'lit/directives/ref'

import { DomClient } from '../clients/dom'
import { InfoViewContext, infoviewContext } from '../contexts/infoview-context'
import { withTwind } from '../twind'
import type { DocumentId } from '../types'

/**
 * View of information about the document, including the currently selected node
 *
 * This is the rightmost panel in both the writer and reader apps. It provides
 * summary information about the document (e.g. authors, summary metrics
 * about AI usage).
 *
 * Uses a `DomClient` instance to maintain the a DOM of the document in sync with
 * its state on the server.
 *
 * It also listens for events emitted from other views indicating the id of the
 * currently selected node and displays a "node card" for that node by setting
 * its class to "show" inside the DOM element. This approach has the advantage
 * over our previous approach of cloning the selected DOM element of staying
 * "live" as changes are made on the server.
 */
@customElement('stencila-info-view')
@withTwind()
export class InfoView extends LitElement {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  @consume({ context: infoviewContext, subscribe: true })
  context: InfoViewContext

  /**
   * A read-only client which updates a (mostly) invisible DOM element when the
   * document changes on the server.
   */
  // @ts-expect-error "dom client is set, but not read"
  private domClient: DomClient

  /**
   * A ref for the invisible element that the `DomClient` updates
   *
   * Used when toggling on/off visibility of nodes within it.
   */
  public domElement: Ref<HTMLElement> = createRef()

  private currentNode: HTMLElement

  /**
   * Override `LitElement.firstUpdated` so that `DomClient` is instantiated _after_ this
   * element has a document `[root]` element in its `renderRoot`.
   */
  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    this.domClient = new DomClient(
      this.doc,
      this.renderRoot.querySelector('[root]') as HTMLElement
    )
  }

  override async update(changedProperties: Map<string, string | boolean>) {
    super.update(changedProperties)
    const { currentNodeId } = this.context

    if (this.currentNode && this.currentNode.id !== currentNodeId) {
      this.currentNode.classList.remove('show')
    }

    if (currentNodeId) {
      this.currentNode = this.domElement.value.querySelector(
        `#${currentNodeId}`
      )
      if (this.currentNode) {
        this.currentNode.classList.add('show')
      }
    }
  }

  // TODO: listen for events from other views for this document
  // for changes to the selected node and set the class of the element
  // in the dom that has that id to "show".

  override render() {
    const domElementClasses = css`
      & *:not(.show) {
        display: none;
      }
    `

    return html`
      <div>
        <div class=${domElementClasses} ${ref(this.domElement)}>
          <stencila-article root></stencila-article>
        </div>
      </div>
    `
  }
}
