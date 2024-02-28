import '@shoelace-style/shoelace/dist/components/icon/icon'

import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import {
  ExecutableStatus,
  executableIcon,
} from '../../../nodes/helpers/node-executables'
import { withTwind } from '../../../twind'

@customElement('stencila-status-gutter-marker')
@withTwind()
class StatusGutterMarkerEl extends LitElement {
  @property({ type: Number })
  defaultLineHeight: number

  status: ExecutableStatus = 'idle'

  protected override render = () => {
    const styles = apply([
      'flex items-center justify-center',
      `w-[${this.defaultLineHeight}px] h-[${this.defaultLineHeight}px]`,
      'bg-white',
      'border rounded',
      'text-lg',
    ])

    return html`<div class=${styles}>${this.renderIcon()}</div>`
  }

  renderIcon = () => {
    const { iconLibrary, icon } = executableIcon(this.status)

    return html`<sl-icon library=${iconLibrary} name=${icon}></sl-icon>`
  }
}

export { StatusGutterMarkerEl }
