import { Component, h, Prop, State } from '@stencil/core'
import { Plugin } from 'stencila'
import { i18n } from '../../../../i18n'
import { client } from '../../../client'
import { capitalize } from '../../utils/stringUtils'
import {
  getAvailablePlugins,
  pluginStore,
} from '../app-settings-plugins/pluginStore'

@Component({
  tag: 'app-settings-plugin-card',
  styleUrl: 'app-settings-plugin-card.css',
  scoped: true,
})
export class AppSettingsPluginCard {
  @Prop() pluginName: string

  @State() plugin?: Plugin

  @State() inProgress: boolean

  componentWillLoad() {
    this.refreshPluginState()
  }

  private refreshPluginState = () =>
    getAvailablePlugins().then(() => {
      this.plugin = pluginStore.plugins.entities[this.pluginName]
    })

  private install = (name: string) => {
    this.inProgress = true

    client.plugins
      .install(name)
      .then(this.refreshPluginState)
      .finally(() => {
        this.inProgress = false
      })
  }

  private uninstall = (name: string) => {
    this.inProgress = true

    client.plugins
      .uninstall(name)
      .then(this.refreshPluginState)
      .finally(() => {
        this.inProgress = false
      })
  }

  private button = () => {
    if (this.plugin?.next) {
      return (
        <stencila-button
          isLoading={this.inProgress}
          icon="refresh"
          size="small"
        >
          {i18n.t('settings.plugins.upgrade')}
        </stencila-button>
      )
    }

    if (this.plugin?.installation) {
      return (
        <stencila-button
          isLoading={this.inProgress}
          color="neutral"
          icon="delete-bin-2"
          onClick={() => this.uninstall(this.pluginName)}
          size="small"
        >
          {i18n.t('settings.plugins.uninstall')}
        </stencila-button>
      )
    }

    return (
      <stencila-button
        isLoading={this.inProgress}
        color="primary"
        icon="download"
        onClick={() => this.install(this.pluginName)}
        size="small"
      >
        {i18n.t('settings.plugins.install')}
      </stencila-button>
    )
  }

  render() {
    return (
      <div class="pluginCard">
        <div class="title">
          <span>
            <h2>{capitalize(this.plugin?.alias ?? this.pluginName)}</h2>
            {this.plugin?.softwareVersion && (
              <span class="meta">v{this.plugin.softwareVersion}</span>
            )}
          </span>

          {this.button()}
        </div>

        <p>{this.plugin?.description}</p>
      </div>
    )
  }
}
