import { property } from 'lit/decorators.js'

import { Executable } from './executable'

/**
 * Abstract base class for web components representing Stencila Schema `CodeExecutable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-executable.md
 */
export abstract class CodeExecutable extends Executable {
  @property()
  code: string = ''

  @property({ attribute: 'programming-language' })
  programmingLanguage?: string

  // properties to track the counts of message types
  @property()
  warnCount: number = 0

  @property()
  errorCount: number = 0

  @property()
  debugCount: number = 0

  /**
   * Resets the message type count properties to 0
   */
  private resetMessageCounts = () => {
    this.errorCount = 0
    this.warnCount = 0
    this.debugCount = 0
  }

  override connectedCallback(): void {
    super.connectedCallback()

    // add observer to watch for slot changes
    const observer = new MutationObserver((mutationList) => {
      mutationList.forEach((m) => {
        if (m.type === 'childList') {
          const slot = this.shadowRoot.querySelector(
            'slot[name="execution-messages"]'
          ) as HTMLSlotElement
          const messages = slot
            .assignedElements()[0]
            .querySelectorAll('stencila-execution-message')
          // reset the message counts, incase of double rendering
          this.resetMessageCounts()
          messages.forEach((m) => {
            // @ts-expect-error 'level property not included in el type'
            if (m.level) {
              // @ts-expect-error 'same as above'
              switch (m.level) {
                case 'Error':
                  this.errorCount += 1
                  return
                case 'Warn':
                case 'Warning':
                  this.warnCount += 1
                  return
                default:
                  this.debugCount += 1
                  return
              }
            }
          })
        }
      })
    })

    observer.observe(this.shadowRoot, { childList: true, subtree: true })
  }
}
