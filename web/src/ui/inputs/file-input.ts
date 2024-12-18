import { html, LitElement } from 'lit'
import { customElement, property, query } from 'lit/decorators'

import { withTwind } from '../../twind'

import '../buttons/icon'

@customElement('stencila-ui-file-input')
@withTwind()
export class UIFileInput extends LitElement {
  @query('input.file-input[hidden]')
  fileInputEl!: HTMLInputElement

  @property({ type: Boolean })
  disabled: boolean = false

  @property({ type: Function })
  fileChangeHandler: (event: Event) => void

  private handleClick() {
    this.fileInputEl.click()
  }

  protected override render() {
    return html`
      <div>
        <stencila-ui-icon-button
          name="filePlus"
          @click=${this.handleClick}
          ?disabled=${this.disabled}
        ></stencila-ui-icon-button>
        <input
          class="file-input"
          type="file"
          @change=${this.fileChangeHandler}
          hidden
          ?disabled=${this.disabled}
        />
      </div>
    `
  }
}
