import type { LabelType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { booleanConverter } from '../utilities/booleanConverter'

import '../ui/nodes/properties/code/code-static'

/**
 * Static version of the CodeChunk web component
 *
 * This is a lightweight alternative for static views that that does not render
 * node and uses Prism.js instead of CodeMirror for syntax highlighting.
 *
 * It only renders:
 *
 * - Code (if isEchoed is true)
 * - Outputs (if isHidden is false)
 * - Captions (for figure/table label types)
 */
@customElement('stencila-code-chunk')
export class CodeChunkStatic extends LitElement {
  @property()
  code: string

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string

  @property({ attribute: 'label-type' })
  labelType?: LabelType

  @property()
  label?: string

  @property({
    attribute: 'is-echoed',
    converter: booleanConverter,
  })
  isEchoed?: boolean

  @property({
    attribute: 'is-hidden',
    converter: booleanConverter,
  })
  isHidden?: boolean

  override render() {
    return html`
      ${this.isEchoed ? this.renderCode() : ''}
      ${this.isHidden ? '' : this.renderOutputs()}
    `
  }

  private renderCode() {
    return html`
      <stencila-ui-node-code-static
        code=${this.code}
        language=${this.programmingLanguage}
      ></stencila-ui-node-code-static>
    `
  }

  private renderOutputs() {
    return html`
      ${this.labelType === 'TableLabel'
        ? html`<caption class="block">
            <slot name="caption"></slot>
          </caption>`
        : ''}

      <slot name="outputs"></slot>

      ${this.labelType === 'FigureLabel'
        ? html`<figcaption><slot name="caption"></slot></figcaption>`
        : ''}
    `
  }
}
