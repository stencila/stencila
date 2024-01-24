import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Person` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md
 */
@customElement('stencila-person')
export abstract class Person extends Entity {
  // TODO: add other properties as needed

  @property({ attribute: 'given-names', type: Array })
  givenNames?: string[]

  @property({ attribute: 'family-names', type: Array })
  familyNames?: string[]

  render() {
    // TODO: improve rendering
    return html`${this.givenNames} ${this.familyNames}`
  }
}
