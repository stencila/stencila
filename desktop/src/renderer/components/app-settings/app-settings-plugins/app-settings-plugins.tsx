import { Component, h, State } from '@stencil/core'
import { i18n } from '../../../../i18n'
import { client } from '../../../client'
import { getAvailablePlugins, pluginStore } from './pluginStore'

@Component({
  tag: 'app-settings-plugins',
  styleUrl: 'app-settings-plugins.css',
  scoped: true,
})
export class AppSettingsPlugins {
  @State() plugins: Plugin[] = []

  @State() inProgress: boolean

  async componentWillLoad() {
    return getAvailablePlugins()
  }

  private checkForUpdates = () => {
    this.inProgress = true

    client.plugins.refresh().finally(() => {
      this.inProgress = false
    })
  }

  render() {
    return (
      <div class="appSettingsPlugins">
        <div class="title">
          <h1>{i18n.t('settings.plugins.title')}</h1>
          <stencila-button
            onClick={this.checkForUpdates}
            size="xsmall"
            color="neutral"
          >
            {i18n.t('settings.plugins.checkUpdates')}
          </stencila-button>
        </div>
        {pluginStore.plugins.ids.map((pluginName) => (
          <app-settings-plugin-card
            pluginName={pluginName}
          ></app-settings-plugin-card>
        ))}
      </div>
    )
  }
}
