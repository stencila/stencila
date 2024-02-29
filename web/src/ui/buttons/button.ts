import { Twind, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * Supplies a "theme" to a button. Specifies:
 * - Initial bg & text
 * - Hover bg & text
 */
export type ButtonTheme = {
  bgDefault: string
  bgHover: string
  textDefault: string
  textHover: string
}

export const BlueTheme = {
  bgDefault: 'blue-700',
  bgHover: 'blue-800',
  textDefault: 'white',
  textHover: 'white',
} as const

const ButtonThemes = {
  blue: BlueTheme,
}

/**
 * UI Button
 *
 * Creates a "submit" style button
 */
@customElement('stencila-ui-button')
@withTwind()
export class UIIconButton extends LitElement {
  @property()
  theme: keyof typeof ButtonThemes = 'blue'

  // Set the type on the `tw` var
  private tw: Twind

  override render() {
    const theme = this.tw.theme()
    const buttonDefault = theme.colors['blue-700'] as string
    const buttonHover = theme.colors['blue-800'] as string
    const textDefault = theme.colors['white'] as string
    const textHover = theme.colors['white'] as string
    const fontSize = '14px'

    const styles = css`
      &::part(base) {
        --sl-input-height-medium: 26px;

        border-radius: 3px;
        border-width: 0px;
        box-shadow: 0px 1px 0px 0px rgba(255, 255, 255, 0.25) inset;
        background-color: ${buttonDefault};
        color: ${textDefault};
        line-height: 0;
        display: flex;
        flex-direction: row;
        font-size: ${fontSize};
        align-items: center;
        justify-content: center;
        font-weight: 500;
        padding: 6px 36px;

        &:hover {
          background-color: ${buttonHover};
          color: ${textHover};
        }
      }

      &::part(label) {
        margin: auto;
        display: contents;
      }
    `

    return html`<sl-button class="${styles}"><slot></slot></sl-button>`
  }
}
