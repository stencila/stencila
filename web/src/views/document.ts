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

    // Append the document menu
    const menu = document.createElement('stencila-document-menu')
    this.appendChild(menu)

    return this
  }
}
