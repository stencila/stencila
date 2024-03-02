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
    console.log('Execute: ', this.nodeId)
    // await fetch(`/~documents/${this.doc}/execute`, {
    //   method: 'POST',
    //   headers: {
    //     'Content-Type': 'application/json',
    //   },
    //   body: JSON.stringify({ nodeId: this.nodeId }),
    // })
  }

  // private interrupt = async (): Promise<void> => {
  //   await fetch(`/~documents/${this.doc}/interrupt`, {
  //     method: 'POST',
  //     headers: {
  //       'Content-Type': 'application/json',
  //     },
  //     body: JSON.stringify({ nodeId: this.nodeId }),
  //   })
  // }

  protected override render = () => {
    const { text, icon } = executableIcon(this.execStatus, this.execRequired)
    const styles = apply([
      'flex items-center justify-center',
      `w-[${this.defaultLineHeight}px] h-[${this.defaultLineHeight}px]`,
      'bg-white',
      'border rounded',
      'text-base',
    ])

    return html`<sl-tooltip placement=${this.tooltipPlacement} content=${text}>
      <button class=${styles} @click=${this.execute}>
        <sl-icon library=${icon.library} name=${icon.name}></sl-icon>
      </button>
    </sl-tooltip>`
  }
}

export { StatusGutterMarkerEl }
