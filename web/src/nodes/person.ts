import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

/**
 * Web component representing a Stencila Schema `Person` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/other/person.md
 */
@customElement('stencila-person')
export abstract class Person extends LitElement {
  // TODO: add other properties as needed

  @property({ attribute: 'given-names', type: Array })
  givenNames?: string[]

  @property({ attribute: 'family-names', type: Array })
  familyNames?: string[]

  render() {
    // TODO: improve rendering
    return html` ${this.givenNames} ${this.familyNames} `
  }
}
