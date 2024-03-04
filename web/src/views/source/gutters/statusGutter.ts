import '@shoelace-style/shoelace/dist/components/icon/icon'
import SlTooltip from '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { ExecutionRequired, ExecutionStatus } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

// import { executableIcon } from '../../../nodes/helpers/node-executables'
import { withTwind } from '../../../twind'

@customElement('stencila-status-gutter-marker')
@withTwind()
class StatusGutterMarkerEl extends LitElement {
  @property({ type: Number })
  defaultLineHeight: number

  @property({ type: String })
  execRequired: ExecutionRequired = 'NeverExecuted'

  @property({ type: String })
  execStatus: ExecutionStatus

  @property({ type: String })
  nodeId: string

  @property({ type: String })
  doc: string

  @property({ type: Number })
  count: number = 0

  tooltipPlacement?: SlTooltip['placement'] = 'top'

  protected override render = () => {
    // const { text, icon } = executableIcon(this.execStatus, this.execRequired)
    const styles = apply([
      'flex items-center justify-center',
      `w-[${this.defaultLineHeight}px] h-[${this.defaultLineHeight}px]`,
      'bg-white',
      'border rounded-full',
      'text-xs leading-none',
    ])

    return html`<sl-tooltip
      placement=${this.tooltipPlacement}
      content=${'times executed: ' + this.count}
    >
      <div class=${styles}><span>${this.count}</span></div>
    </sl-tooltip>`
  }
}

export { StatusGutterMarkerEl }
