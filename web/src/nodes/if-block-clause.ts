import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import '../ui/nodes/card'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `IfBlockClause` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-block-clause.md
 */
@customElement('stencila-if-block-clause')
@withTwind()
export class IfBlockClause extends CodeExecutable {
  /**
   * Whether the clause is the active branch of the parent `IfBlock`
   *
   * Note: this is not a boolean property, it is a string that looks
   * like a boolean :)
   */
  @property({ attribute: 'is-active' })
  isActive?: 'true' | 'false'

  /**
   * Whether the clause has any content
   *
   * This state is used to determine whether to render placeholder
   * text if there is no content for the clause.
   *
   * @see this.renderContent()
   */
  @state()
  private hasContent = false

  /**
   * A mutation observer to update the `hasContent` state when
   * the `content` slot changes
   */
  private contentObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `content` slot
   */
  private onContentSlotChange(event: Event) {
    // Get the slot element
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    // Set current state
    this.hasContent = contentElem.childElementCount > 0

    // Update the state when the slot is mutated
    this.contentObserver = new MutationObserver(() => {
      this.hasContent = contentElem.childElementCount > 0
    })
    this.contentObserver.observe(contentElem, {
      childList: true,
    })
  }

  override render() {
    const { colour, borderColour, textColour } = nodeUi('IfBlock')

    const isActive = this.isActive == 'true'

    const siblings = [...this.parentElement.children]
    const index = siblings.findIndex((elem) => elem === this)
    let label
    if (index === 0) {
      label = 'if'
    } else if (index == siblings.length - 1 && this.code.length == 0) {
      label = 'else'
    } else {
      label = 'elif'
    }

    const borderPosition = index == siblings.length - 1 ? '0' : 'b'

    return html`
      <!-- TODO: this header should hidden when the parent IfBlock is collapsed -->
      <div
        class="px-1 py-3 bg-[${colour}] border-${borderPosition} border-[${borderColour}]"
      >
        <!-- TODO: add icon, preferably different for if, elif and else -->

        <span class="${isActive
          ? `rounded ring-2 ring-[${textColour}] ring-offset-4 ring-offset-[${colour}]`
          : ''} font-bold font-mono text-[${textColour}] m-3"
          >${label}</span>

        <!-- TODO: improve appearance of this code: rounded borders?, minimum width or full width-->
        <stencila-ui-node-code
          type="IfBlock"
          code=${this.code}
          .code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          read-only
          no-gutters
          containerClasses="inline-block border border-[${borderColour}]"
          class=${label === 'else' ? 'hidden' : ''}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-code>

        <!-- TODO: use icon for lang if available, if not then the name in a pill or something -->
        <span class="font-mono text-[${textColour}]">${this.programmingLanguage ?? ''}</span>

        <!-- TODO: Add a chevron to collapse/expand the content, regardless of if active or not -->
      </div>

      <div class="p-1 ${isActive ? '' : 'hidden'}">
        <p class="text-center text-grey-400 italic" contenteditable="false">
          ${this.hasContent ? '' : 'No content'}
        </p>
        <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
      </div>
    `
  }
}
