import type { Validator, Node } from '@stencila/types'
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

  constructor() {
    super()

    this.addEventListener('input', (event: Event) => {
      const target = event.target as HTMLInputElement

      const value = target.value

      // TODO: Handle different types of values
      // using target.valueAsNumber and target.valueAsDate

      this.patchNode({
        op: 'replace',
        id: this._id,
        path: 'value',
        value,
      })
    })
  }

  override render() {
    return html`
      <label for="${this.name}">${this.label ?? this.name}</label>
      <input name="${this.name}" />
    `
  }

  /**
   * This accessor exists to allow us to get around the eslint rule: wc/no-constructor-attributes (as called in the constructor) - via: plugin:wc/recommended
   */
  get _id() {
    return this.id
  }
}
