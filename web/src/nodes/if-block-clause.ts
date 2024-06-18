import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import '../ui/nodes/card'

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
   * Whether the clause has any content
   *
   * This state is used to determine whether to render placeholder
   * text if there is no content.
   *
   * @see this.renderContent()
   */
  // @state()
  // private hasContent = false

  /**
   * A mutation observer to update the `hasContent` state when
   * the `content` slot changes
   */
  // private contentObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `content` slot
   */
  // private onContentSlotChange(event: Event) {
  //   // Get the slot element
  //   const contentElem = (event.target as HTMLSlotElement).assignedElements({
  //     flatten: true,
  //   })[0]

  //   // Set current state
  //   this.hasContent = contentElem.childElementCount > 0

  //   // Update the state when the slot is mutated
  //   this.contentObserver = new MutationObserver(() => {
  //     this.hasContent = contentElem.childElementCount > 0
  //   })
  //   this.contentObserver.observe(contentElem, {
  //     childList: true,
  //   })
  // }

  /**
   * will be true if this clause is the current 'path' of the parent `IfBlock`
   */
  @property({ type: Boolean, attribute: 'is-active' })
  isActive: boolean

  override render() {
    return html`
      <stencila-ui-block-on-demand type="IfBlockClause" view="dynamic">
        <div slot="body" class="h-full">
          <stencila-ui-node-authors type="IfBlockClause">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-code
            type="IfBlockClause"
            code=${this.code}
            language=${this.programmingLanguage}
            read-only
          >
          </stencila-ui-node-code>
          <stencila-ui-node-execution-messages
            type="IfBlockClause"
            message-count=${this.messageCount}
            warning-count=${this.warningCount}
            error-count=${this.errorCount}
          >
            <slot name="execution-messages"></slot>
          </stencila-ui-node-execution-messages>
        </div>
        <div slot="content">
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  // override render() {
  //   return html` <div>${this.renderHeader()} ${this.renderContent()}</div> `
  // }

  // private renderHeader() {
  //   return html` <div contenteditable="false">${this.renderMessages()}</div> `
  // }

  // private renderContent() {
  //   return html`
  //     <div>
  //       <p class="text-grey-400" contenteditable="false">
  //         ${this.hasContent ? '' : 'No content'}
  //       </p>
  //       <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
  //     </div>
  //   `
  // }
}
