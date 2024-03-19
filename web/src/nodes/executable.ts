import {
  AutomaticExecution,
  ExecutionRequired,
  ExecutionStatus,
  ExecutionTag,
} from '@stencila/types'
import { property, state } from 'lit/decorators.js'

import { Entity } from './entity'
import { ExecutionMessage } from './execution-message'

/**
 * Abstract base class for web components representing Stencila Schema `Executable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md
 */
export abstract class Executable extends Entity {
  @property({ attribute: 'auto-exec' })
  autoExec?: AutomaticExecution

  @property({ attribute: 'execution-tags', type: Array })
  executionTags?: ExecutionTag[]

  @property({ attribute: 'execution-count', type: Number })
  executionCount?: number

  @property({ attribute: 'execution-required' })
  executionRequired?: ExecutionRequired

  @property({ attribute: 'execution-status' })
  executionStatus?: ExecutionStatus

  @property({ attribute: 'execution-ended', type: Number })
  executionEnded?: number

  @property({ attribute: 'execution-duration', type: Number })
  executionDuration?: number

  /**
   * The number of messages in the `execution-messages` slot
   */
  @state()
  messageCount: number = 0

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
          this.messageCount = messages.length
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
