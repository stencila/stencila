import '@shoelace-style/shoelace/dist/components/icon/icon'
import { AuthorRoleName } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `AuthorRole` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author-role.md
 */
@customElement('stencila-author-role')
@withTwind()
export class AuthorRole extends Entity {
  @property({ attribute: 'role-name' })
  roleName: AuthorRoleName

  // TODO: Add more properties when they are available in the schema
  // e.g date and format

  override render() {
    return html`<div class="my-1">
      <div class="text-xs">${this.roleName}</div>
      <slot></slot>
    </div>`
  }
}
