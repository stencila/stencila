import { MutationController } from '@lit-labs/observers/mutation-controller'
import { AuthorRoleName, NodeType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/author'

import { Entity } from './entity'
import { Organization } from './organization'
import { Person } from './person'
import { SoftwareApplication } from './software-application'

/**
 * Web component representing a Stencila Schema `AuthorRole` node
 *
 * To unify an `AuthorRole` with other types that can be an author (e.g `Person`)
 * into a `<stencila-ui-author>` it is necessary to reach into the default slot
 * of this component and extract `name` etc from it.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author-role.md
 */
@customElement('stencila-author-role')
@withTwind()
export class AuthorRole extends Entity {
  @property({ attribute: 'role-name' })
  roleName: AuthorRoleName

  @property({ attribute: 'last-modified', type: Number })
  lastModified: number

  @state()
  private type: NodeType

  @state()
  private $id: string

  @state()
  private name: string

  /**
   * An observer to watch this element, including slotted children,
   * and call `getAuthor` on any mutations
   */
  private observer = new MutationController(this, {
    config: { subtree: true, attributes: true },
    callback: () => this.getAuthor(),
  })

  /**
   * Get the inner `author` of this author role and extract `type`
   * and `name` state properties. Changes to these will trigger a re-render.
   */
  private getAuthor() {
    const slot: HTMLSlotElement = this.renderRoot.querySelector('slot')
    if (slot) {
      const inner = slot.assignedElements({ flatten: true })[0]

      if (inner.tagName.toLowerCase() === 'stencila-person') {
        const person = inner as Person
        this.type = 'Person'
        this.name = `${(person.givenNames ?? []).join(' ')} ${(person.familyNames ?? []).join(' ')}`
      } else if (
        inner.tagName.toLowerCase() === 'stencila-software-application'
      ) {
        const app = inner as SoftwareApplication
        this.type = 'SoftwareApplication'
        this.$id = app.$id
        this.name = app.name
      } else if (inner.tagName.toLowerCase() === 'stencila-organization') {
        const app = inner as Organization
        this.type = 'Organization'
        this.name = app.name
      }
    }
  }

  override render() {
    return html`
      <stencila-ui-author
        type=${this.type}
        _id=${this.$id}
        name=${this.name}
        role=${this.roleName}
        timestamp=${this.lastModified}
      ></stencila-ui-author>

      <div hidden><slot></slot></div>
    `
  }
}
