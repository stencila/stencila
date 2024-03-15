import { MessageLevel } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html, css } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

@customElement('stencila-ui-node-execution-message')
@withTwind()
export class ExecutionMessage extends LitElement {
  @property()
  level: MessageLevel

  @property({ attribute: 'error-type' })
  errorType?: string

  @property({ type: Boolean })
  messageCollapsed: boolean = true

  private toggleCollapse = () => {
    this.messageCollapsed = !this.messageCollapsed
  }

  private getMarginSpecs = (): { colour: string; icon: string } => {
    switch (this.level) {
      case 'Error':
        return { colour: 'pink-900', icon: 'x-circle' }
      // @ts-expect-error '"Warning" is being used, but is not included in the type union'
      case 'Warning':
      case 'Warn':
        return { colour: 'orange-500', icon: 'exclamation-circle' }
      case 'Info':
        return { colour: 'green-900', icon: 'info-circle' }
      case 'Debug':
      case 'Trace':
      default:
        return { colour: 'green-900', icon: 'question-circle' }
    }
  }

  static override styles = css`
    slot[name='message']::slotted(pre) {
      text-wrap: wrap;
    }
    slot[name='stack-trace']::slotted(pre) {
      overflow-x: auto;
    }
  `

  override render() {
    const { colour, icon } = this.getMarginSpecs()
    // styles for the margin element
    const marginStyles = apply([
      'flex-shrink-0',
      'min-w-12',
      'p-1',
      `bg-${colour}/50`,
    ])

    // styles for the outer message element
    const msgContainerStyles = apply(['flex-grow', 'px-0.5', 'overflow-hidden'])

    // styles for the expandable message container
    const msgBodyStyles = apply([
      'h-full',
      'text-xs leading-1 font-mono',
      this.messageCollapsed ? 'max-h-8' : 'max-h-[1000px]',
      'transition-max-h duration-200',
      'overflow-y-hidden',
    ])

    return html`
      <div class="flex p-1">
        <div class=${marginStyles}>
          <sl-icon class="text-xs text-green-100" name=${icon}></sl-icon>
          <stencila-chevron-button
            default-pos="right"
            position=${this.messageCollapsed ? 'right' : 'down'}
            custom-class="text-xs"
            colour="green-100"
            .clickEvent=${() => this.toggleCollapse()}
          ></stencila-chevron-button>
        </div>
        <div class=${msgContainerStyles}>
          <div class=${msgBodyStyles}>
            <slot name="message"></slot>
            <slot name="stack-trace"></slot>
          </div>
        </div>
      </div>
    `
  }
}
