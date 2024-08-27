import { provide } from '@lit/context'
import { LitElement, html } from 'lit'
import { state } from 'lit/decorators'

import {
  DocPreviewContext,
  NodeChipState,
  documentPreviewContext,
} from '../../document/context'

export abstract class DocumentPreviewBase extends LitElement {
  @provide({ context: documentPreviewContext })
  @state()
  protected context: DocPreviewContext = {
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

  protected renderPreviewMenu() {
    return html`
      <preview-menu
        ?visible=${this.showMenu}
        ?show-authorship-highlight=${this.context.showAllAuthorshipHighlight}
        ?show-author-provenance=${this.context.showAuthorProvenance}
        node-chip-state=${this.context.nodeChipState}
      ></preview-menu>
    `
  }
}
