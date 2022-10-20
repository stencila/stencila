import { css, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/input/input'

import '../base/tag'
import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'
import SlInput from '@shoelace-style/shoelace/dist/components/input/input'
import { TW } from 'twind'

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
        border-color: rgba(221, 214, 254, var(--tw-border-opacity));
      }
    `,
  ]

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
   */
  @property({ reflect: true })
  select: string

  /**
   * Whether the `Include.content` property is visible
   */
  @state()
  protected isExpanded: boolean = true

  /**
   * Whether the `Include.content` property has content
   */
  @state()
  private hasContent: boolean = false

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

  protected renderSourceInput(tw: TW) {
    const replace = (event: Event): boolean => {
      const input = event.target as SlInput
      if (input.reportValidity()) {
        this.source = (event.target as HTMLInputElement).value
        this.emitReplaceOperations('source')
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
      ?disabled=${this.isReadOnly()}
      @sl-change=${replace}
      @sl-blur=${replace}
      @keypress=${(event: KeyboardEvent) => {
        if (event.key == 'Enter' && event.ctrlKey) {
          event.preventDefault()
          if (replace(event)) {
            this.compile()
          }
        }
      }}
    ></sl-input>`
  }

  protected renderSelectInput(tw: TW) {
    const replace = (event: Event): boolean => {
      const input = event.target as SlInput
      if (input.reportValidity()) {
        this.select = (event.target as HTMLInputElement).value
        this.emitReplaceOperations('select')
        return true
      }
      return false
    }

    return html`<sl-input
      class=${tw`min-w-0 w-full`}
      size="small"
      value=${this.select}
      ?disabled=${this.isReadOnly()}
      @sl-change=${replace}
      @sl-blur=${replace}
      @keypress=${(event: KeyboardEvent) => {
        if (event.key == 'Enter' && event.ctrlKey) {
          event.preventDefault()
          if (replace(event)) {
            this.compile()
          }
        }
      }}
    ></sl-input>`
  }

  protected renderExpandButton(tw: TW, color: string) {
    return html`<stencila-icon-button
      name="chevron-right"
      color=${color}
      adjust=${`ml-2 rotate-${this.isExpanded ? 90 : 0} transition-transform`}
      @click=${() => {
        this.isExpanded = !this.isExpanded
      }}
      @keydown=${(event: KeyboardEvent) => {
        if (
          event.key == 'Enter' ||
          (event.key == 'ArrowUp' && this.isExpanded) ||
          (event.key == 'ArrowDown' && !this.isExpanded)
        ) {
          event.preventDefault()
          this.isExpanded = !this.isExpanded
        }
      }}
    >
    </stencila-icon-button>`
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
    const color = 'purple'
    return html`<div
      part="base"
      class=${tw`my-4 rounded border(& ${color}-200) overflow-hidden`}
    >
      <div
        part="header"
        class=${tw`flex items-center bg-${color}-100 p-1 font(mono bold) text(sm ${color}-800)`}
      >
        <span class=${tw`flex items-center text-base mr-2`}>
          <stencila-icon name="box-arrow-in-right"></stencila-icon>
        </span>
        <span class=${tw`mr-2`}>include</span>
        ${this.renderSourceInput(tw)}
        <span class=${tw`mx-2`}>select</span>
        ${this.renderSelectInput(tw)} ${this.renderExpandButton(tw, color)}
      </div>
      ${this.renderErrorsContainer(tw, color)}
      ${this.renderContentContainer(tw, color)}
    </div>`
  }
}
