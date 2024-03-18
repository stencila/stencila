import '@shoelace-style/shoelace/dist/components/icon/icon'
import { ArrayValidator, NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { nodeUi } from '../ui/nodes/icons-and-colours'

/**
 * Web component representing an item within a Stencila Schema `Datatable` node
 */
@customElement('stencila-datatable-column')
export class DatatableColumn extends LitElement {
  @property()
  name: string

  @property({ type: Object })
  validator?: ArrayValidator

  override render() {
    const itemsValidatorType = this.validator?.itemsValidator?.type
    const itemsType = itemsValidatorType
      ? itemsValidatorType.replace('Validator', '')
      : ''
    const { icon, iconLibrary } = nodeUi(itemsType as NodeType)

    return html` <span class="items-center flex">
      <sl-icon
        library=${iconLibrary}
        name=${icon}
        class=${`pr-2 text-xl`}
      ></sl-icon>
      <span class="font-bold">${this.name}</span>
    </span>`
  }
}
