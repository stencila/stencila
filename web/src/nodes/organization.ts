import '@shoelace-style/shoelace/dist/components/icon/icon'

import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/properties/author'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Organization` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/organization.md
 */
@customElement('stencila-organization')
@withTwind()
export class Organization extends Entity {
  @property()
  name: string

  override render() {
    return html`<stencila-ui-node-author
      type="Organization"
      name=${this.name}
    ></stencila-ui-node-author>`
  }
}
