import { provide } from '@lit/context'
import { LitElement, html } from 'lit'
import { state } from 'lit/decorators'

import {
  DocumentContext,
  NodeChipState,
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
    nodeChipState: 'hover-only',
    showAuthorProvenance: false,
  }

  /**
   * Whether the menu/toggle button is visible
   */
  @state()
  protected showMenu: boolean = false

  /**
   * Override so that this component has a Light DOM so that
   * theme styles apply to it.
   * Apply menu related event listeners to the root
   */
  protected override createRenderRoot() {
    // event listeners for toggle menu visibility

    this.addEventListener('mouseenter', () => {
      this.showMenu = true
    })

    this.addEventListener('mouseleave', () => {
      this.showMenu = false
    })

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
      'update-nodecard-state',
      (e: Event & { detail: NodeChipState }) => {
        this.context = {
          ...this.context,
          nodeChipState: e.detail,
        }
      }
    )

    return this
  }

  protected renderDocumentMenu() {
    return html`<stencila-document-menu></stencila-document-menu>`
  }
}
