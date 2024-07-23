import '@shoelace-style/shoelace/dist/components/icon/icon'
import { MessageLevel } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html, css } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import '../../buttons/chevron'
import { executionMessageUI } from '../icons-and-colours'

@customElement('stencila-ui-node-execution-message')
@withTwind()
export class ExecutionMessage extends LitElement {
  // Height for message body when collapsed
  // private BASE_HEIGHT = 32 as const

  @property()
  level: MessageLevel

  @property()
  message: string

  @property({ attribute: 'error-type' })
  errorType?: string

  @property({ attribute: 'stack-trace' })
  stackTrace?: string

  // @state()
  // private messageCollapsed: boolean = true

  // private toggleCollapse = () => {
  //   this.messageCollapsed = !this.messageCollapsed
  // }

  static override styles = css`
    pre {
      text-wrap: wrap;
    }
  `

  override render() {
    const { colour } = executionMessageUI(this.level)

    // styles for the margin element
    const marginStyles = apply(['w-2', `bg-${colour}`])

    // styles for the expandable message container
    const msgBodyStyles = apply([
      'h-full',
      'ml-1',
      'text-xs leading-1 font-sans',
    ])

    return html`
      <div class="flex p-1">
        <div class=${marginStyles}></div>
        <div class="flex-grow ml-2">
          <div class=${msgBodyStyles}>
            <pre>${this.message}</pre>
            <pre>${this.stackTrace}</pre>
          </div>
        </div>
      </div>
    `
  }
}
