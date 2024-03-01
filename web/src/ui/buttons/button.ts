import { Twind, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

import { ButtonThemes } from './themes'

/**
 * UI Button
 *
 * Creates a "submit" style button
 */
@customElement('stencila-ui-button')
@withTwind()
export class UIIconButton extends LitElement {
  /**
   * The name of the theme to use.
   */
  @property()
  theme: keyof typeof ButtonThemes = 'blue-solid'

  @property()
  clickEvent: (e: Event) => void | undefined

  @property({ type: Boolean })
  disabled = false

  // Set the type on the `tw` var
  private tw: Twind

  override render() {
    const theme = ButtonThemes[this.theme]
    const fontSize = '14px'
    const isInline = this.theme.includes('inline')

    const baseStyles = css`
      &::part(base) {
        --sl-input-height-medium: 26px;

        border-radius: 3px;
        background-color: ${this.getTWColour(theme.bgDefault)};
        color: ${this.getTWColour(theme.textDefault)};
        line-height: 0;
        display: flex;
        flex-direction: row;
        font-size: ${fontSize};
        align-items: center;
        justify-content: center;
        font-weight: 500;
        padding: 6px ${theme.px ?? '36px'};
        text-decoration: ${theme.textDecorationDefault ?? 'none'};
        height: 30px;

        &:hover {
          background-color: ${this.getTWColour(theme.bgHover)};
          color: ${this.getTWColour(theme.textHover)};
          text-decoration: ${theme.textDecorationHover ?? 'none'};
        }
      }

      &::part(label) {
        margin: auto;
        display: contents;
      }
    `

    const themeStyles = isInline
      ? this.getInlineStyles()
      : this.getSolidStyles()

    return html`<sl-button
      class="${baseStyles} ${themeStyles}"
      @click=${this.clickEvent}
      .disabled=${this.disabled}
      ><slot></slot
    ></sl-button>`
  }

  private getTWColour(colourName: string, fallback: string = 'black') {
    const theme = this.tw.theme()
    return (theme.colors[colourName] ?? theme.colors[fallback]) as string
  }

  private getSolidStyles() {
    return css`
      &::part(base) {
        border-width: 0;
        box-shadow: 0px 1px 0px 0px rgba(255, 255, 255, 0.25) inset;
      }
    `
  }

  private getInlineStyles() {
    const theme = ButtonThemes[this.theme]

    return css`
      &::part(base) {
        border-width: 1px;
        border-color: ${this.getTWColour(theme.borderDefault)};

        &:hover {
          border-color: ${this.getTWColour(theme.borderHover)};
        }
      }
    `
  }
}
