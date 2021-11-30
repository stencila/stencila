import { Component, h, State } from '@stencil/core'
import { LogLevel, LogMessage } from 'electron-log'
import { v4 as uuidv4 } from 'uuid'
import { i18n } from '../../../../i18n'
import { CHANNEL } from '../../../../preload/channels'
import { client } from '../../../client'

const defaultFilters = {
  silly: true,
  debug: true,
  info: true,
  verbose: true,
  warn: true,
  error: true,
}

@Component({
  tag: 'app-logs-list',
  styleUrl: 'app-logs-list.css',
  scoped: true,
})
export class AppLogsList {
  @State() logs: LogMessage[] = []

  @State() logLevelFilter: Record<LogLevel, boolean> = defaultFilters

  private setFilter = (level: LogLevel): void => {
    this.logLevelFilter = {
      ...this.logLevelFilter,
      [level]: !this.logLevelFilter[level],
    }
  }

  private resetFilters = (): void => {
    this.logLevelFilter = defaultFilters
  }

  private filteredLogs = (): LogMessage[] => {
    return this.logs.filter((log) => this.logLevelFilter[log.level])
  }

  private clearLogs = (e: Event) => {
    e.preventDefault()
    this.logs = []
  }

  async componentWillLoad() {
    const { value: logs } = await client.app.logs.get()
    this.logs = logs

    window.api.receive(CHANNEL.LOGS_PRINT, (logItem) => {
      this.logs = [...this.logs, logItem as LogMessage]
    })
  }

  private renderFilterCheckbox = (isEnabled: boolean) =>
    isEnabled ? (
      <stencila-icon icon="checkbox"></stencila-icon>
    ) : (
      <stencila-icon icon="checkbox-blank"></stencila-icon>
    )

  render() {
    const visibleLogs = this.filteredLogs()

    return (
      <div class="logsList">
        <main>
          <div class="title">
            <div>
              <h1>{i18n.t('logs.title')}</h1>

              <p class="secondaryText">
                {i18n.t('logs.filteredResults', {
                  visibleCount: visibleLogs.length,
                  totalCount: this.logs.length,
                })}
              </p>
            </div>

            <div>
              <stencila-menu>
                <stencila-button
                  icon="filter"
                  size="xsmall"
                  color="neutral"
                  slot="toggle"
                >
                  {i18n.t('logs.filter')}
                </stencila-button>

                <stencila-menu-item onClick={this.resetFilters}>
                  Everything
                </stencila-menu-item>

                <stencila-menu-item
                  onClick={(e) => {
                    e.preventDefault()
                    this.setFilter('verbose')
                    this.setFilter('debug')
                  }}
                >
                  {this.renderFilterCheckbox(this.logLevelFilter.verbose)}
                  Verbose
                </stencila-menu-item>
                <stencila-menu-item
                  onClick={(e) => {
                    e.preventDefault()
                    this.setFilter('info')
                  }}
                >
                  {this.renderFilterCheckbox(this.logLevelFilter.info)}
                  Info
                </stencila-menu-item>
                <stencila-menu-item
                  onClick={(e) => {
                    e.preventDefault()
                    this.setFilter('warn')
                  }}
                >
                  {this.renderFilterCheckbox(this.logLevelFilter.warn)}
                  Warning
                </stencila-menu-item>
                <stencila-menu-item
                  onClick={(e) => {
                    e.preventDefault()
                    this.setFilter('error')
                  }}
                >
                  {this.renderFilterCheckbox(this.logLevelFilter.error)}
                  Error
                </stencila-menu-item>
              </stencila-menu>
              <stencila-button
                icon="delete-bin-2"
                onClick={this.clearLogs}
                size="xsmall"
                color="neutral"
              >
                {i18n.t('logs.clear')}
              </stencila-button>
            </div>
          </div>

          {this.logs.length === 0 ? (
            <h2>{i18n.t('logs.empty')}</h2>
          ) : (
            visibleLogs.map((log) => (
              <app-logs-item logMessage={log} key={uuidv4()}></app-logs-item>
            ))
          )}
        </main>
      </div>
    )
  }
}
