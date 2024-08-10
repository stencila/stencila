import '@shoelace-style/shoelace/dist/components/icon/icon'
import { ArrayValidator, NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Entity } from './entity'

/**
 * Web component representing an item within a Stencila Schema `Datatable` node
 */
@customElement('stencila-datatable-column')
@withTwind()
export class DatatableColumn extends Entity {
  @property()
  name: string

  @property({ type: Object })
  validator?: ArrayValidator

  override render() {
    const itemsValidatorType = this.validator?.itemsValidator?.type
    const itemsType = itemsValidatorType
      ? itemsValidatorType.replace('Validator', '')
      : ''
    const { icon } = nodeUi(itemsType as NodeType)

    return html`
      <div class="flex justify-start items-center">
        <stencila-ui-icon
          name=${icon}
          class=${`pr-0.5 text-base`}
        ></stencila-ui-icon>
        <span class="font-bold">${this.name}</span>
      </div>
    `
  }
}
