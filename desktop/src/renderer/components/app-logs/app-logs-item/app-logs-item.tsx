import { Component, h, Host, Prop } from '@stencil/core'
import { IconNames } from '@stencila/components'
import { LogMessage } from 'electron-log'

const formatMessage = (message: string): string => {
  const separator = ' |%c '
  if (message.includes(separator)) {
    return message.split(separator)[1] ?? message
  }
  return message
}

@Component({
  tag: 'app-logs-item',
  styleUrl: 'app-logs-item.css',
  scoped: true,
})
export class AppLogsItem {
  @Prop() logMessage!: LogMessage

  private getIcon(severity: LogMessage['level']): IconNames {
    switch (severity) {
      case 'error':
        return 'forbid'
      case 'warn':
        return 'error-warning'
      default:
        return 'information'
    }
  }

  render() {
    return (
      <Host class={{ [this.logMessage.level]: true }}>
        <div>
          <stencila-icon
            icon={this.getIcon(this.logMessage.level)}
          ></stencila-icon>
          <code>{formatMessage(this.logMessage.data[0])}</code>
        </div>

        <span class="meta">
          <code>{this.logMessage.date.toLocaleTimeString()}</code>
        </span>
      </Host>
    )
  }
}
