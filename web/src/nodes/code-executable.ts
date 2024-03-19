import { property, state } from 'lit/decorators.js'

import { Executable } from './executable'
import { ExecutionMessage } from './execution-message'

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

  /**
   * The number of `Warning` level messages in the `execution-messages` slot
   */
  @state()
  warningCount: number = 0

  /**
   * The number of `Error` level messages in the `execution-messages` slot
   */
  @state()
  errorCount: number = 0

  override connectedCallback(): void {
    super.connectedCallback()

    // Add observer to watch for slot changes
    const observer = new MutationObserver((mutationList) => {
      mutationList.forEach((mutation) => {
        if (mutation.type === 'childList') {
          const slot = this.shadowRoot.querySelector(
            'slot[name="execution-messages"]'
          ) as HTMLSlotElement
          const messages = (slot
            .assignedElements()[0]
            ?.querySelectorAll('stencila-execution-message') ??
            []) as ExecutionMessage[]

          // Reset the message counts
          this.warningCount = 0
          this.errorCount = 0

          messages.forEach((message: ExecutionMessage) => {
            switch (message.level) {
              case 'Warning':
                this.warningCount += 1
                return
              case 'Error':
              case 'Exception':
                this.errorCount += 1
                return
            }
          })
        }
      })
    })

    observer.observe(this.shadowRoot, { childList: true, subtree: true })
  }
}
