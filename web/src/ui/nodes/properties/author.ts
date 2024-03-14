import { AuthorRoleName } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

/**
 * A component for displaying an `Author` within the `authors` property of nodes
 *
 * In the Stencila Schema [`Author`](https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/author.md)
 * is a "Union type for things that can be an author of a CreativeWork or other type":
 *
 * - Person
 * - Organization
 * - SoftwareApplication
 * - AuthorRole
 *
 * Note: the above is NOT a list of the properties of an author, it is a list of the types that can
 * be an author. Do not let this confuse you :)
 *
 * `Author` is implemented as a UI component here so that we have a uniform way of displaying all the different
 * types which can be considered an author and which can thus be in the `authors` property. This component
 * exposes properties that the various types can bind to. Properties are used rather than slots to maintain
 * typing and because they are all simple atomic values.
 */
@customElement('stencila-ui-node-author')
@withTwind()
export class UINodeAuthor extends LitElement {
  /**
   * The type of node that is the author
   *
   * Might be useful for determining default icons etc.
   */
  @property()
  type: 'Person' | 'Organization' | 'SoftwareApplication'

  /**
   * The id of the author
   *
   * Currently only available for `SoftwareApplication` authors. The intension is
   * to use this to be able to fetch more information about the authors (e.g. affiliation)
   * from some canonical source in the document. But currently this has not been finalized
   * (or even started).
   */
  @property({ attribute: '_id' })
  $id?: string

  /**
   * The name of the author
   *
   * Should be available for all author types, for a `Person` by concatenating
   *  `givenNames` and `familyNames`.
   */
  @property()
  name: string

  /**
   * The role that the author has
   *
   * Only available for `AuthorRole` authors.
   */
  @property({ attribute: 'role' })
  roleName?: AuthorRoleName

  /**
   * The timestamp of the last modification made by the author in a particular role
   *
   * Only available for `AuthorRole` authors.
   */
  @property({ type: Number })
  timestamp?: number

  override render() {
    return html`<div class="my-1">
      ${this.type} | ${this.$id} | ${this.name} | ${this.roleName} |
      ${this.timestamp}
    </div>`
  }
}
