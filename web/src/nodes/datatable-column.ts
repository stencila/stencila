import { ArrayValidator } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/icons/icon'

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
    return html`<slot></slot>`
  }
}
