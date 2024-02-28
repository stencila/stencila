import '@shoelace-style/shoelace/dist/components/icon/icon'
import SlTooltip from '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { ExecutionRequired, ExecutionStatus } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { executableIcon } from '../../../nodes/helpers/node-executables'
import { withTwind } from '../../../twind'

@customElement('stencila-status-gutter-marker')
@withTwind()
class StatusGutterMarkerEl extends LitElement {
  @property({ type: Number })
  defaultLineHeight: number

  execRequired: ExecutionRequired = 'NeverExecuted'
  execStatus: ExecutionStatus

  tooltipPlacement?: SlTooltip['placement'] = 'top'

  @property({ type: String })
  nodeId: string

  @property({ type: String })
  doc: string

  private execute = async (): Promise<void> => {
    await fetch(`/~documents/${this.doc}/execute`, {
      method: 'POST',
      body: JSON.stringify({ nodeId: this.nodeId }),
    })
  }

  private interupt = async (): Promise<void> => {
    await fetch(`/~documents/${this.doc}/interupt`, {
      method: 'POST',
      body: JSON.stringify({ nodeId: this.nodeId }),
    })
  }

  protected override render = () => {
    const styles = apply([
      'flex items-center justify-center',
      `w-[${this.defaultLineHeight}px] h-[${this.defaultLineHeight}px]`,
      'bg-white',
      'border rounded',
      'text-base',
    ])

    return html`<sl-tooltip
      placement=${this.tooltipPlacement}
      content=${`Execute`}
    >
      <button class=${styles} @click=${this.execute}>
        ${this.renderIcon()}
      </button>
    </sl-tooltip>`
  }

  renderIcon = () => {
    const { iconLibrary, icon } = executableIcon(this.execRequired)

    return html`<sl-icon library=${iconLibrary} name=${icon}></sl-icon>`
  }
}

export { StatusGutterMarkerEl }
