import { provide } from '@lit/context'
import { LitElement, html } from 'lit'
import { state } from 'lit/decorators'

import {
  DocPreviewContext,
  documentPreviewContext,
} from '../../../contexts/preview-context'

export abstract class DocumentPreviewBase extends LitElement {
  @provide({ context: documentPreviewContext })
  @state()
  protected context: DocPreviewContext = {
    showAllToggleChips: false,
    showAllAuthorshipHighlight: false,
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

    // event listeners for preview context `CustomEvent`

    this.addEventListener('toggle-card-chips', () => {
      this.context = {
        ...this.context,
        showAllToggleChips: !this.context.showAllToggleChips,
      }
    })

    this.addEventListener('toggle-authorship-highlight', () => {
      console.log('event captured')
      this.context = {
        ...this.context,
        showAllAuthorshipHighlight: !this.context.showAllAuthorshipHighlight,
      }
    })

    return this
  }

  protected renderPreviewMenu() {
    return html`
      <preview-menu
        ?visible=${this.showMenu}
        ?show-toggle-chips=${this.context.showAllToggleChips}
        ?show-authorship-highlight=${this.context.showAllAuthorshipHighlight}
      ></preview-menu>
    `
  }
}
