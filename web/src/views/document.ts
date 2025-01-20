import { provide } from '@lit/context'
import { LitElement } from 'lit'
import { state } from 'lit/decorators'

import {
  DocumentContext,
  NodeMarkerState,
  documentContext,
} from '../ui/document/context'

/**
 * A base class for document views which provides a document menu
 */
export abstract class DocumentView extends LitElement {
  @provide({ context: documentContext })
  @state()
  protected context: DocumentContext = {
    showAllAuthorshipHighlight: false,
    nodeMarkerState: 'hover-only',
    showAuthorProvenance: false,
  }

  menu: HTMLElement

  observer: MutationObserver

  private observeRootElement: MutationCallback = (_mutations) => {
    const rootNode = this.querySelector('[root]')

    if (rootNode) {
      const rootTag = rootNode.tagName.toLowerCase()
      if (!this.menu && rootTag === 'stencila-article') {
        this.menu = document.createElement('stencila-document-menu')
        this.appendChild(this.menu)
      } else if (this.menu && rootTag !== 'stencila-article') {
        // remove menu in unlikely event of root node changing from article to another type
        this.menu.remove()
      }
    }
  }

  /**
   * Override so that this component has a Light DOM so that
   * theme styles apply to it.
   * Apply menu related event listeners to the root
   */
  protected override createRenderRoot() {
    // event listeners for preview context `CustomEvent`s

    this.addEventListener('toggle-authorship-highlight', () => {
      this.context = {
        ...this.context,
        showAllAuthorshipHighlight: !this.context.showAllAuthorshipHighlight,
      }
    })

    this.addEventListener('toggle-author-provenance', () => {
      this.context = {
        ...this.context,
        showAuthorProvenance: !this.context.showAuthorProvenance,
      }
    })

    this.addEventListener(
      'update-nodemarker-state',
      (e: Event & { detail: NodeMarkerState }) => {
        this.context = {
          ...this.context,
          nodeMarkerState: e.detail,
        }
      }
    )

    this.observer = new MutationObserver(this.observeRootElement)

    this.observer.observe(this, { childList: true })

    return this
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    this.observer.disconnect()
  }
}
