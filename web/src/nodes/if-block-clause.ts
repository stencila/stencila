import { ContextConsumer } from '@lit/context'
import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { IconName } from '../ui/icons/icon'
import { EntityContext, entityContext } from '../ui/nodes/entity-context'
import { nodeUi } from '../ui/nodes/icons-and-colours'
import { booleanConverter } from '../utilities/booleanConverter'

import '../ui/animation/collapsible'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/content-placeholder'
import '../ui/nodes/properties/programming-language'

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
   */
  @property({ attribute: 'is-active', converter: booleanConverter })
  isActive?: boolean = false

  /**
   * Consumer for the parent `IfBlock` node's entity context
   *
   * Used to check whether the card for the `IfBlock` that this clause
   * is a member of is open or not. If it is open, the content of
   * all clauses should be visible. If it is closed, only the content
   * of the active clauses should be visible.
   */
  @state()
  private ifBlockConsumer: ContextConsumer<{ __context__: EntityContext }, this>

  /**
   * Whether the clause is expanded (i.e. its content is shown even if it is inactive)
   */
  @state()
  private isExpanded: boolean = false

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

  protected override updated(changedProperties: PropertyValues): void {
    super.updated(changedProperties)

    // Initialize the `ifBlockConsumer` object
    if (!this.ifBlockConsumer) {
      this.ifBlockConsumer = new ContextConsumer(this, {
        context: entityContext,
        subscribe: true,
      })
    }
  }

  override render() {
    return html`
      ${this.ifBlockConsumer?.value?.cardOpen ? this.renderHeader() : ''}

      <stencila-ui-collapsible-animation
        class=${this.isActive || this.isExpanded ? 'opened' : ''}
      >
        ${this.renderContent()}
      </stencila-ui-collapsible-animation>
    `
  }

  protected renderHeader() {
    const { colour, borderColour, textColour } = nodeUi('IfBlock')

    const siblings = [...this.parentElement.children]
    const index = siblings.findIndex((elem) => elem === this)

    let label
    let iconName: IconName
    if (index === 0) {
      label = 'if'
      iconName = 'ifClause'
    } else if (index == siblings.length - 1 && this.code.length == 0) {
      label = 'else'
      iconName = 'elseClause'
    } else {
      label = 'elif'
      iconName = 'elifClause'
    }

    const readOnly =
      ['Running', 'Pending'].includes(this.executionStatus) || !this.hasRoot()

    const expanded = this.isActive || this.isExpanded

    const headerStyle = apply([
      'px-3 py-1.5 flex items-center',
      `text-[${textColour}] bg-[${colour}] border-[${borderColour}]`,
      index == 0 ? '' : 'border-t',
      !expanded ? '' : 'border-b',
    ])

    const iconStyles = apply([
      `text-xl text-${textColour}`,
      this.isActive
        ? `rounded ring-2 ring-[${textColour}] ring-offset-4 ring-offset-[${colour}]`
        : '',
    ])

    return html`
      <div class=${headerStyle}>
        <stencila-ui-icon name=${iconName} class=${iconStyles}>
        </stencila-ui-icon>

        <span class="font-bold font-mono ml-3 min-w-[3rem]"> ${label} </span>

        <stencila-ui-node-code
          type="IfBlock"
          code=${this.code}
          .code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          execution-required=${this.executionRequired}
          ?read-only=${readOnly}
          single-line
          node-id=${this.id}
          no-gutters
          container-classes="inline-block w-full border border-[${borderColour}] rounded overflow-hidden"
          class=${label === 'else'
            ? 'hidden'
            : 'flex-grow flex items-center mr-1 max-w-[80%]'}
        >
          <slot name="execution-messages" slot="messages"></slot>
        </stencila-ui-node-code>

        ${this.programmingLanguage
          ? html`<stencila-ui-node-programming-language
              programming-language=${this.programmingLanguage}
            ></stencila-ui-node-programming-language>`
          : ''}

        <stencila-ui-chevron-button
          class="ml-auto"
          default-pos=${expanded ? 'down' : 'left'}
          slot="right-side"
          custom-class="flex items-center ml-3"
          ?disabled=${this.isActive === true}
          .clickEvent=${() => (this.isExpanded = !this.isExpanded)}
        ></stencila-ui-chevron-button>
      </div>
    `
  }

  protected renderContent() {
    const styles = apply([
      this.ifBlockConsumer?.value?.cardOpen ? 'px-2 pb-4' : '',
      this.hasContent ? '' : 'pt-4',
    ])

    return html`<div class=${styles}>
      ${this.hasContent
        ? ''
        : html`<stencila-ui-node-content-placeholder></stencila-ui-node-content-placeholder>`}
      <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
    </div>`
  }
}
