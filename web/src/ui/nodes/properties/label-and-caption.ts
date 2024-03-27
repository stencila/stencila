import { LabelType, NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/simple'

/**
 * A component for displaying the `label` and `caption`
 * properties of `Table`, `Figure` and `CodeChunk` nodes
 */
@customElement('stencila-ui-node-label-and-caption')
@withTwind()
export class UINodeExecutionDuration extends LitElement {
  @property()
  type: NodeType

  /**
   * The type of label. Only for `CodeChunk` nodes.
   */
  @property({ attribute: 'label-type' })
  labelType?: LabelType

  @property()
  label?: string

  override render() {
    let text = ''

    if (this.label) {
      text +=
        this.labelType === 'FigureLabel'
          ? 'Figure'
          : this.labelType === 'TableLabel'
            ? 'Table'
            : this.type
      text += ` ${this.label} `
    }

    return html`<div class="px-4">
      <span class="font-bold">${text}</span>
      <slot name="caption"></slot>
    </div>`
  }
}
