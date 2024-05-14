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
    this.addEventListener('mouseenter', () => {
      this.showMenu = true
    })

    this.addEventListener('mouseleave', () => {
      this.showMenu = false
    })

    this.addEventListener('toggle-card-chips', () => {
      this.context = {
        ...this.context,
        showAllToggleChips: !this.context.showAllToggleChips,
      }
    })
    return this
  }

  override render() {
    return html`
      <slot></slot>
      <preview-menu
        ?visible=${this.showMenu}
        ?show-toggle-chips=${this.context.showAllToggleChips}
      ></preview-menu>
    `
  }
}
