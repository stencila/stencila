import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/author'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Person` node
 *
 * Note: currently this only implements a very few of the properties
 * of a `Person`. Properties such as `affiliations` are missing form this component at this time.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md
 */
@customElement('stencila-person')
@withTwind()
export class Person extends Entity {
  @property({ attribute: 'given-names', type: Array })
  givenNames?: string[]

  @property({ attribute: 'family-names', type: Array })
  familyNames?: string[]

  override render() {
    const name = `${(this.givenNames ?? []).join(' ')} ${(this.familyNames ?? []).join(' ')}`

    return html`<stencila-ui-author
      type="Person"
      name=${name}
    ></stencila-ui-author>`
  }
}
