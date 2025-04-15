import { MutationController } from '@lit-labs/observers/mutation-controller'
import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../twind'

@customElement('stencila-ui-article-references')
@withTwind()
export class ArticleReferences extends LitElement {
  /**
   * A mutation controller used to determine whether to add a "References" heading
   *
   * @see onSlotChange
   */
  // @ts-expect-error is never read
  private mutationController: MutationController

  /**
   * Initialize the mutation controller when the slot changes
   */
  onSlotChange({ target: slot }: Event) {
    const referencesElem = (slot as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]
    if (!referencesElem) {
      return
    }

    this.mutationController = new MutationController(this, {
      target: referencesElem,
      config: {
        childList: true,
      },
      callback: () => {
        if (
          referencesElem.querySelectorAll('stencila-reference').length > 0 &&
          referencesElem.querySelectorAll('h1').length == 0
        ) {
          const heading = document.createElement('stencila-heading')
          heading.setAttribute('depth', '1')
          heading.setAttribute('level', '1')
          heading.innerHTML = '<h1 slot="content">References</h1>'
          referencesElem.prepend(heading)
        }
      },
    })
  }

  override render() {
    return html`<slot @slotchange=${this.onSlotChange}></slot>`
  }
}
