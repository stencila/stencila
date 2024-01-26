import { apply, css } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { TWLitElement } from '../twind'

/**
 * UI selector
 *
 * A selector that updates some display portion of the UI
 */
@customElement('stencila-ui-selector-button')
export class UISelector extends TWLitElement {
  /**
   * Label for the selector
   */
  @property()
  label: string = ''

  override render() {
    const styles = apply([
      'text-base font-bold',
      'leading-none',
      'select-none',
      'appearance-none ',
      'min-w-fit',
      'py-2 px-4',
      'bg-white',
      'border-b-4 border-b-transparent',
      'transition-all ease-in-out',
      'flex',
      'items-center',
      'group-hover:text-brand-blue group-hover:border-b-brand-blue',
    ])

    const classes = css`
      &::part(base) {
        border: none;
        padding: 0;
        outline: none;

        &:hover {
          background: none;
        }
      }

      &::part(label) {
        padding-left: 0;
      }
    `

    return html` <sl-button slot="trigger" class="${styles} ${classes}" caret
      >${this.label}</sl-button
    >`
  }
}
