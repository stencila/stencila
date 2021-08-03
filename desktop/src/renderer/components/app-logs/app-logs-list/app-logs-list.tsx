import { Component, h, State } from '@stencil/core'
import { LogMessage } from 'electron-log'
import { v4 as uuidv4 } from 'uuid'
import { i18n } from '../../../../i18n'
import { CHANNEL } from '../../../../preload/channels'
import { client } from '../../../client'

@Component({
  tag: 'app-logs-list',
  styleUrl: 'app-logs-list.css',
  scoped: true,
})
export class AppLogsList {
  @State() logs: LogMessage[] = []

  private clearLogs = (e: Event) => {
    this.logs = []
  }

  async componentWillLoad() {
    const { value } = await client.app.logs.get()
    this.logs = value

    window.api.receive(CHANNEL.LOGS_PRINT, (logItem) => {
      this.logs = [...this.logs, logItem as LogMessage]
    })
  }

  render() {
    return (
      <div class="logsList">
        <main>
          <div class="title">
            <h1>{i18n.t('logs.title')}</h1>
            <stencila-button
              icon="delete-bin-2"
              onClick={this.clearLogs}
              size="xsmall"
              color="neutral"
            >
              {i18n.t('logs.clear')}
            </stencila-button>
          </div>

          {this.logs.length === 0 ? (
            <h2>{i18n.t('logs.empty')}</h2>
          ) : (
            this.logs.map((log) => (
              <app-logs-item logMessage={log} key={uuidv4()}></app-logs-item>
            ))
          )}
        </main>
      </div>
    )
  }
}
