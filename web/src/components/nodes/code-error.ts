import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { twSheet } from '../utils/css'
import StencilaEntity from './entity'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `CodeError` node
 *
 * @slot message The `CodeError.errorMessage` property
 * @slot stacktrace The `CodeError.stackTrace` property
 */
@customElement('stencila-code-error')
export default class CodeError extends StencilaEntity {
  static styles = sheet.target

  static color = 'red'

  /**
   * The `CodeError.errorType` property
   *
   * This is a property, rather than a slot, because it should be a simple, unformatted string,
   * and having it as a property makes it easier to do things like have different icons for
   * different error types.
   */
  @property({ attribute: 'error-type' })
  errorType?: string

  /**
   * Whether the error has any stacktrace
   */
  @state()
  private hasStacktrace = false

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
  private isStacktraceVisible = false

  /**
   * Get the icon icon for the error type
   *
   * At present just returns a single icon but in future this may vary
   * with the error type.
   */
  protected getIcon() {
    return 'exclamation-octagon'
  }

  protected render() {
    const inline = this.parentElement?.tagName.toLowerCase() === 'span'

    const viewStacktraceButton =
      this.hasStacktrace && !inline
        ? html`<span
            class=${tw`flex items-center p-1 rounded-full outline-none focus:ring(2 ${CodeError.color}-200) cursor-pointer`}
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
          class=${tw`max-w-xs bg-${CodeError.color}-50 px-1 font(mono) text(sm ${CodeError.color}-700)`}
        >
          <stencila-icon
            name=${this.getIcon()}
            class=${tw`inline-block align-middle`}
          ></stencila-icon>
          <span class=${tw`font-bold`}>${this.errorType ?? 'Error'}</span>
          <code part="message"><slot name="message"></slot></code>
        </span>`
      : html`<div
          part="base"
          class=${tw`bg-${CodeError.color}-50 overflow-x-auto font(mono) text(sm ${CodeError.color}-700)`}
        >
          <div part="header" class=${tw`flex justify-between p-2 pr-1`}>
            <span class=${tw`flex items-center`}>
              <stencila-icon
                name=${this.getIcon()}
                class=${tw`mr-2`}
              ></stencila-icon>
              <span class=${tw`font-bold`}>${this.errorType ?? 'Error'}</span>
            </span>
            ${viewStacktraceButton}
          </div>

          <pre
            part="message"
            class=${tw`flex justify-between p-2 pr-1`}
          ><slot name="message"></slot></pre>

          <pre
            part="stacktrace"
            class=${tw`border(t ${
              CodeError.color
            }-100) p-2 pt-4 overflow-x-auto ${
              this.isStacktraceVisible || 'hidden'
            }`}
          ><slot
          name="stacktrace"
          @slotchange=${(event: Event) => this.onStacktraceSlotChange(event)}
        ></slot></pre>
        </div>`
  }
}
