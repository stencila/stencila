import '@shoelace-style/shoelace/dist/components/icon/icon'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Person` node
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
    return html`<div class="my-1">
      <span class="items-center flex leading-3 text-base">
        <sl-icon name="person-circle" class="pr-2"></sl-icon>
        ${this.givenNames} ${this.familyNames}
      </span>
    </div>`
  }
}
