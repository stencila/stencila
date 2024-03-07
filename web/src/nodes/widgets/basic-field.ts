import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

@customElement('stencila-basic-node-field')
@withTwind()
export class BasicNodeField extends LitElement {
  @property({ attribute: 'icon-name' })
  iconName: string

  @property({ attribute: 'icon-library' })
  iconLibrary: 'stencila' | 'default' = 'stencila'

  override render() {
    return html`
      <div class="flex flex-row w-full mb-4">
        <div class="pt-0.5">
          <sl-icon
            name=${this.iconName}
            library=${this.iconLibrary}
            class="text-base text-black"
          ></sl-icon>
        </div>
        <div class="grow ml-4">
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
