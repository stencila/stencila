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
import '../nodes'
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
    const { currentNodeId, currentParentNodes } = this.context
    if (this.currentNode && this.currentNode.id !== currentNodeId) {
      this.currentNode.classList.remove('active-node')
      const showing = this.domElement.value.querySelector('.show')
      if (showing) {
        showing.classList.remove('show')
      }
      this.domElement.value.querySelectorAll('[active-child]').forEach((el) => {
        el.removeAttribute('active-child')
      })
    }
    if (currentNodeId) {
      this.currentNode = this.domElement.value.querySelector(
        `#${currentNodeId}`
      )
      this.currentNode.classList.add('active-node')
      // if node is at top level in doc append .show
      if (this.currentNode) {
        if (!currentParentNodes) {
          this.currentNode.classList.add('show')
        } else {
          // ID of the highest ancestor node
          currentParentNodes.forEach((id, idx, arr) => {
            const el = this.domElement.value.querySelector(`#${id}`)

            if (el) {
              el.setAttribute('active-child', arr[idx - 1] ?? currentNodeId)

              // append .show to top most element of the node branch
              if (idx === arr.length - 1) {
                el.classList.add('show')
              }
            }
          })
        }
      }
      console.log('currently active node: ', this.currentNode)
    }
  }

  override render() {
    const domElementClasses = css`
      width: 100%;
      height: 100%;

      /* hide all non-active node trees at the root level */
      & [root] > section > *:not(.show) {
        display: none;
      }

      /* set the [root] element to block display to allow full height */
      & [root] {
        display: block;
        height: 100%;
      }

      /* set all elements to block with full height to allow full space */
      & [root],
      slot[name='content']::slotted(*),
      slot[name='items']::slotted(*),
      slot[name='clauses']::slotted(*),
      [slot='content'],
      [slot='items'],
      [slot='clauses'],
      [active-child] {
        display: block;
        height: 100%;
      }

      /* make sure the active node is visible */
      & .active-node {
        visibility: visible;
      }

      /*
        hide previous and following siblings from the dom 
        to avoid white space
      */
      & *:has(~ .active-node),
      *:has(~ [active-child]),
      .active-node ~ *,
      [active-child] ~ * {
        display: none;
      }
    `

    return html`
      <div class="h-full">
        <div class=${domElementClasses} ${ref(this.domElement)}>
          <stencila-article root></stencila-article>
        </div>
      </div>
    `
  }
}
