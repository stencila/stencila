import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI Editor Tab
 *
 * The tab for a document displayed in the main toolbar (above the editor).
 */
@customElement('stencila-ui-editor-tab')
@withTwind()
export class UIEditorTab extends LitElement {
  /**
   * Changing the active property, changes the display of the tab.
   */
  @property({ type: Boolean })
  active: boolean = false

  override render() {
    const classes = apply([
      'text-sm font-medium leading-none',
      'text-neutral-900',
      !this.active ? 'cursor-pointer' : 'cursor-default',
      'block',
      'w-fit',
      'select-none',
      'pl-4 pr-16 py-2',
      'mr-auto',
      '-mb-[1px]',
      this.active ? 'bg-white' : 'bg-grey-200',
      'rounded-t-md',
      'border',
      'border-grey-200',
      this.active && 'border-b-white',
      !this.active && 'hover:underline',
    ])

    return html`<div class=${classes}>
      <slot></slot>
    </div>`
  }
}
