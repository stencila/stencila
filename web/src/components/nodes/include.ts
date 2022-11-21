import { css, html, PropertyValueMap } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import SlInput from '@shoelace-style/shoelace/dist/components/input/input'

import '../base/tag'
import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'
import { TW } from 'twind'
import { isCodeWriteable } from '../../mode'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `Include` node and the base
 * for the derived `Call` node
 *
 * @slot content The `Include.content` property
 * @slot errors The `Executable.errors` property
 */
@customElement('stencila-include')
export default class StencilaInclude extends StencilaExecutable {
  static styles = [
    sheet.target,
    css`
      sl-input::part(base) {
        font-family: monospace;
      }
    `,
  ]

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  /**
   * The `Include.source` property
   */
  @property({ reflect: true })
  source: string

  /**
   * The `Include.mediaType` property
   */
  @property({ reflect: true })
  mediaType: string

  /**
   * The `Include.select` property
   *
   * Uses a different name to avoid clash with `Entity.select()` method.
   */
  @property({ attribute: 'select', reflect: true })
  selectQuery: string

  /**
   * Whether the `Include.content` property has content
   */
  @state()
  private hasContent = false

  /**
   * An observer to update `hasContent`
   */
  private contentObserver: MutationObserver

  /**
   * Handle a change to the `content` slot
   *
   * Initializes `hasContent` and a `MutationObserver` to watch for changes
   * to the number of elements in the slot and change `hasContent` accordingly.
   */
  private onContentSlotChange(event: Event) {
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    this.hasContent = contentElem.childElementCount > 0

    this.contentObserver = new MutationObserver(() => {
      this.hasContent = contentElem.childElementCount > 0
    })
    this.contentObserver.observe(contentElem, {
      childList: true,
    })
  }

  protected renderSourceInput(tw: TW, action: 'compile' | 'execute') {
    const readOnly = !isCodeWriteable()

    const replace = (event: Event): boolean => {
      const input = event.target as SlInput
      if (input.reportValidity()) {
        const source = (event.target as HTMLInputElement).value
        this.changeProperty('source', source)
        return true
      }
      return false
    }

    return html`<sl-input
      class=${tw`min-w-0 w-full`}
      size="small"
      placeholder="path/to/file.ext"
      required="true"
      value=${this.source}
      ?disabled=${readOnly}
      @focus=${() => this.deselect()}
      @mousedown=${(event: MouseEvent) => {
        this.deselect()
        event.stopPropagation()
      }}
      @sl-change=${replace}
      @sl-blur=${replace}
      @keypress=${(event: KeyboardEvent) => {
        if (event.key == 'Enter' && event.ctrlKey) {
          event.preventDefault()
          if (replace(event)) {
            action == 'compile' ? this.compile() : this.execute()
          }
        }
      }}
    ></sl-input>`
  }

  protected renderSelectInput(tw: TW, action: 'compile' | 'execute') {
    const readOnly = !isCodeWriteable()

    const replace = (event: Event): boolean => {
      const input = event.target as SlInput
      if (input.reportValidity()) {
        // Note: can not use `changeProperty` here because
        // using 'selectQuery' as name of 'select' property to
        // avoid a clash with the `select()` method
        const value = (event.target as HTMLInputElement).value
        this.selectQuery = value
        this.emitOp({
          type: 'Replace',
          address: ['select'],
          items: 1,
          length: 1,
          value,
        })
        return true
      }
      return false
    }

    return html`<sl-input
      class=${tw`min-w-0 w-full mr-1`}
      size="small"
      value=${this.selectQuery}
      ?disabled=${readOnly}
      @focus=${() => this.deselect()}
      @mousedown=${(event: MouseEvent) => {
        this.deselect()
        event.stopPropagation()
      }}
      @sl-change=${replace}
      @sl-blur=${replace}
      @keypress=${(event: KeyboardEvent) => {
        if (event.key == 'Enter' && event.ctrlKey) {
          event.preventDefault()
          if (replace(event)) {
            action == 'compile' ? this.compile() : this.execute()
          }
        }
      }}
    ></sl-input>`
  }

  protected renderErrorsContainer(tw: TW, color: string) {
    return html`<div
      part="errors"
      class=${tw`border(t ${color}-200) ${this.hasErrors || 'hidden'}`}
    >
      <slot
        name="errors"
        @slotchange=${(event: Event) => this.onErrorsSlotChange(event)}
      ></slot>
    </div>`
  }

  protected renderContentContainer(tw: TW, color: string) {
    return html`<div
      part="content"
      class=${tw`border(t ${color}-200) p-2 ${this.isExpanded || 'hidden'}`}
    >
      ${!this.hasContent
        ? html`<p class=${tw`text(center gray-300)`}>No content</p>`
        : ''}
      <slot
        name="content"
        @slotchange=${(event: Event) => this.onContentSlotChange(event)}
      ></slot>
    </div>`
  }

  protected render() {
    const toggleSelected = () => this.toggleSelected()

    return html`<div
      part="base"
      class=${tw`my-4 rounded overflow-hidden whitespace-normal border(& ${
        StencilaInclude.color
      }-200) ${this.selected ? `ring-1` : ''}`}
      @mousedown=${toggleSelected}
    >
      <div
        part="header"
        class=${tw`flex items-center bg-${StencilaInclude.color}-50 p-1
                   font(mono bold) text(sm ${StencilaInclude.color}-700)`}
      >
        <span class=${tw`flex items-center text-base mr-2`}>
          <stencila-icon name="box-arrow-in-right"></stencila-icon>
        </span>
        <span class=${tw`mr-2`}>include</span>
        ${this.renderSourceInput(tw, 'compile')}
        <span class=${tw`mx-2`}>select</span>
        ${this.renderSelectInput(tw, 'compile')}
        ${this.renderExpandButton(tw, StencilaInclude.color)}
      </div>

      ${this.renderErrorsContainer(tw, StencilaInclude.color)}
      ${this.renderContentContainer(tw, StencilaInclude.color)}

      <div
        part="footer"
        class=${tw`grid justify-items-end items-center bg-${StencilaInclude.color}-50
                       border(t ${StencilaInclude.color}-200) p-1 text(sm ${StencilaInclude.color}-700)`}
      >
        ${this.renderDownloadButton(
          StencilaInclude.formats,
          StencilaInclude.color
        )}
      </div>
    </div>`
  }
}
