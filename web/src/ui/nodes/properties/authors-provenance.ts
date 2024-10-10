import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { withTwind } from '../../../twind'
import { DocumentContext, documentContext } from '../../document/context'
import { nodeUi } from '../icons-and-colours'

import '../../animation/collapsible'

/**
 * UI Authors and Provenance
 *
 * A collapsible section displayed at the top of the page to show authors &
 * provenance.
 */
@customElement('stencila-ui-authors-provenance')
@withTwind()
export class UIAuthorsProvenance extends LitElement {
  @consume({ context: documentContext, subscribe: true })
  @state()
  context: DocumentContext

  @state()
  collapsed: boolean = true

  /**
   * ref used to manage mouse events.
   */
  // private buttonRef: Ref<HTMLDivElement> = createRef()

  protected override render() {
    const { textColour, colour, borderColour } = nodeUi('Article')

    const contentStyles = apply([
      'p-4 text-sans',
      `bg-[${colour}]`,
      `text-[${textColour}]`,
      `border border-[${borderColour}] rounded`,
      'transition-all ease-in duration-200',
    ])

    return html`
      <div
        class=${`${this.context.showAuthorProvenance ? 'mb-8' : 'mb-0'} transition-all ease-in duration-200 group pointer-events-none`}
      >
        <stencila-ui-collapsible-animation
          class=${`pointer-events-auto ${this.context.showAuthorProvenance ? 'opened' : ''}`}
        >
          <div class=${contentStyles}>
            <slot></slot>
          </div>
        </stencila-ui-collapsible-animation>
      </div>
    `
  }
}
