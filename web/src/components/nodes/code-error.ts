import { html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { twSheet } from '../utils/css'
import StencilaEntity from './entity'

const { tw, sheet } = twSheet()

@customElement('stencila-code-error')
export default class CodeError extends StencilaEntity {
  static styles = sheet.target

  /**
   * Whether the error has any stacktrace
   */
  @state()
  private hasStacktrace: boolean = false

  /**
   * An observer to update `hasStacktrace`
   */
  private stacktraceObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the stacktrace slot
   */
  onStacktraceSlotChange(event: Event) {
    const stacktraceElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]
    this.hasStacktrace = (stacktraceElem.textContent?.length ?? 0) > 0

    this.stacktraceObserver = new MutationObserver(() => {
      this.hasStacktrace = (stacktraceElem.textContent?.length ?? 0) > 0
    })
    this.stacktraceObserver.observe(stacktraceElem, {
      characterData: true,
    })
  }

  /**
   * Whether any stack trace is visible to the user
   */
  @state()
  private isStacktraceVisible: boolean = false

  render() {
    const inline =
      this.parentElement?.tagName.toLowerCase() === 'span' ? true : false

    const viewStacktraceButton =
      this.hasStacktrace && !inline
        ? html`<span
            class=${tw`flex items-center p-1 rounded-full outline-none focus:ring(2 red-200) cursor-pointer`}
            tabindex="0"
            @keydown=${(event: KeyboardEvent) => {
              if (
                event.key == 'Enter' ||
                (event.key == 'ArrowUp' && this.isStacktraceVisible) ||
                (event.key == 'ArrowDown' && !this.isStacktraceVisible)
              ) {
                event.preventDefault()
                this.isStacktraceVisible = !this.isStacktraceVisible
              }
            }}
            @click=${() =>
              (this.isStacktraceVisible = !this.isStacktraceVisible)}
          >
            <stencila-icon name="list"></stencila-icon>
            <stencila-icon
              class=${tw`rotate-${
                this.isStacktraceVisible ? '90' : '0'
              } transition-transform`}
              name="chevron-right"
            ></stencila-icon>
          </span>`
        : ''

    return inline
      ? html`<span
          part="base"
          class=${tw`inline-block max-w-xs bg-red-50 p-1 font(mono) text(sm red-800)`}
        >
          <span part="type" class=${tw`font(bold)`}>
            <slot name="type"></slot>
          </span>
          <code part="message" class=${tw`ml-1`}
            ><slot name="message"></slot
          ></code>
        </span>`
      : html`<div
          part="base"
          class=${tw`bg-red-50 overflow-x-auto font(mono) text(sm red-800)`}
        >
          <div part="header" class=${tw`flex justify-between p-2 pr-1`}>
            <span part="type" class=${tw`font(bold)`}>
              <slot name="type"></slot>
            </span>
            ${viewStacktraceButton}
          </div>

          <pre
            part="message"
            class=${tw`flex justify-between p-2 pr-1`}
          ><slot name="message"></slot></pre>

          <pre
            part="stacktrace"
            class=${tw`border(t red-100) p-2 pt-4 overflow-x-auto ${
              this.isStacktraceVisible || 'hidden'
            }`}
          ><slot
          name="stacktrace"
          @slotchange=${(event: Event) => this.onStacktraceSlotChange(event)}
        ></slot></pre>
        </div>`
  }
}
