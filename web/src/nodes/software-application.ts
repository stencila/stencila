import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/properties/author'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `SoftwareApplication` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/software-application.md
 */
@customElement('stencila-software-application')
@withTwind()
export class SoftwareApplication extends Entity {
  @property({ attribute: '@id' })
  $id?: string

  @property()
  name: string

  @property()
  version?: string

  override render() {
    return html`<stencila-ui-node-author
      type="SoftwareApplication"
      _id=${this.$id}
      name=${this.name}
      details=${`v${this.version}`}
    ></stencila-ui-node-author>`
  }
}
