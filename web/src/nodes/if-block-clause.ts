import { consume } from '@lit/context'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import '../ui/nodes/card'
import { withTwind } from '../twind'
import { AvailableLanguages } from '../types'
import { EntityContext, entityContext } from '../ui/nodes/context'
import { nodeUi } from '../ui/nodes/icons-and-colours'
import { AvailableLanguagesMixin } from '../ui/nodes/mixins/language'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `IfBlockClause` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-block-clause.md
 */
@customElement('stencila-if-block-clause')
@withTwind()
export class IfBlockClause extends AvailableLanguagesMixin(CodeExecutable) {
  /**
   * Whether the clause is the active branch of the parent `IfBlock`
   *
   * Note: this is not a boolean property, it is a string that looks
   * like a boolean :)
   */
  @property({ attribute: 'is-active' })
  isActive?: 'true' | 'false'

  @state()
  private isFolded: boolean = true

  /**
   * consumer for the parent `IfBlock` node's entity context
   * used to check the card status of the
   */
  @consume({ context: entityContext, subscribe: true })
  @state()
  private ifBlockConsumer: EntityContext

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

  protected override update(changedProperties: PropertyValues): void {
    super.update(changedProperties)

    if (changedProperties.has('ifBlockConsumer')) {
      // if card is closed only active path stays open
      if (!this.ifBlockConsumer.cardOpen) {
        this.isFolded = this.isActive === 'false'
      }
    }
  }

  override render() {
    return html`
      ${this.ifBlockConsumer.cardOpen ? this.renderHeader() : ''}
      <stencila-ui-collapsible-animation
        class=${!this.isFolded ? 'opened' : ''}
      >
        <div class="p-3">
          <p class="text-center text-grey-400 italic" contenteditable="false">
            ${this.hasContent ? '' : 'No content'}
          </p>
          <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
        </div>
      </stencila-ui-collapsible-animation>
    `
  }

  protected renderHeader() {
    const { colour, borderColour, textColour } = nodeUi('IfBlock')

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

    const borderPosition = index === siblings.length - 1 ? '0' : 'b'

    return html`
      <div
        class="p-3 flex items-center text-[${textColour}] bg-[${colour}] border-${borderPosition} border-[${borderColour}]"
      >
        <sl-icon
          name=${label === 'else' ? 'node-minus' : 'node-plus'}
          library="default"
          class="text-lg"
        >
        </sl-icon>
        <span class="font-bold font-mono mx-3 min-w-[4rem]">
          <span
            class="${this.isActive === 'true'
              ? `rounded ring-2 ring-[${textColour}] ring-offset-4 ring-offset-[${colour}]`
              : ''}"
            >${label}</span
          >
        </span>

        <!-- TODO: improve appearance of this code: rounded borders?, minimum width or full width-->
        <stencila-ui-node-code
          type="IfBlock"
          code=${this.code}
          .code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          read-only
          no-gutters
          containerClasses="inline-block border border-[${borderColour}] rounded overflow-hidden mr-2"
          class=${label === 'else' ? 'hidden' : 'flex items-center'}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-code>

        <!-- TODO: use icon for lang if available, if not then the name in a pill or something -->
        ${this.renderLang()}

        <stencila-chevron-button
          class="ml-auto"
          default-pos=${this.isFolded ? 'left' : 'down'}
          slot="right-side"
          custom-class="flex items-center"
          .clickEvent=${() => (this.isFolded = !this.isFolded)}
        ></stencila-chevron-button>
      </div>
    `
  }

  protected renderLang() {
    const { textColour } = nodeUi('IfBlock')
    if (this.programmingLanguage) {
      if (this.programmingLanguage in this.languages) {
        const lang =
          this.languages[this.programmingLanguage as AvailableLanguages]
        return html`
          <sl-icon
            class="text-lg"
            name=${lang.icon[0]}
            library=${lang.icon[1]}
          ></sl-icon>
        `
      } else {
        return html`<span
          class="font-mono text-sm px-2 border border-[${textColour}] rounded-full"
          >${this.programmingLanguage}</span
        > `
      }
    }
    return ''
  }
}
