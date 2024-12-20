import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/option/option'
import '../icons/icon'

type SelectOption = {
  value: string
  label?: string
  icon?: string
}

@customElement('ui-select-input')
@withTwind()
export class UISelect extends LitElement {
  @property({ type: Boolean })
  multi: boolean = false

  @property({ type: Boolean })
  clearable: boolean = false

  @property({ type: Array })
  options: SelectOption[] = []

  @state()
  value: string

  protected override render(): unknown {
    return html`
      <sl-select
        ?multiple=${this.multi}
        ?clearable=${this.clearable}
        size="small"
        max-options-visible="2"
      >
        ${this.options.map((opt) => {
          return html`
            <sl-option value=${opt.value}>
              ${opt.icon
                ? html`<stencila-ui-icon
                    slot="prefix"
                    class="text-base"
                    name=${opt.icon}
                  ></stencila-ui-icon>`
                : ''}
              ${opt.label ?? opt.value}
            </sl-option>
          `
        })}
      </sl-select>
    `
  }
}
