import { AuthorRoleName } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/properties/author'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `AuthorRole` node
 *
 * To unify an `AuthorRole` with other types that can be an author (e.g `Person`)
 * into a `<stencila-ui-node-author>` it is necessary to reach into the default slot
 * of this component and extract `name` etc from it.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author-role.md
 */
@customElement('stencila-author-role')
@withTwind()
export class AuthorRole extends Entity {
  @property({ attribute: 'role-name' })
  roleName: AuthorRoleName

  @property()
  format: string

  @property({ attribute: 'last-modified', type: Number })
  lastModified: number

  @property()
  type: 'Person' | 'Organization' | 'SoftwareApplication'

  @property()
  _id: string

  @property()
  name: string

  @property()
  details: string

  override render() {
    return html`
      <stencila-ui-node-author
        type=${this.type}
        _id=${this._id}
        name=${this.name}
        role=${this.roleName}
        format=${this.format}
        timestamp=${this.lastModified}
        details=${this.details}
      ></stencila-ui-node-author>
    `
  }
}
