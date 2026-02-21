import type { Node, Validator } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { Executable } from './executable'

@customElement('stencila-parameter')
export class Parameter extends Executable {
  @property()
  name: string

  @property()
  label?: string

  @property()
  value?: Node

  @property()
  default?: Node

  @property({ type: Object })
  validator?: Validator

  override render() {
    return html`
      <label for="${this.name}">${this.label ?? this.name}</label>
      <input name="${this.name}" />
    `
  }
}
