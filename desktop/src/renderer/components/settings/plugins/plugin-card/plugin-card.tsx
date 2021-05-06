import { Component, h, Prop, State } from '@stencil/core'
import { plugins } from 'stencila'
import { CHANNEL } from '../../../../../preload/index'
import { getAvailablePlugins, pluginStore } from '../pluginStore'

@Component({
  tag: 'plugin-card',
  styleUrl: 'plugin-card.css',
  scoped: true,
})
export class AppPlugins {
  @Prop() pluginName: string

  @State() plugin?: plugins.Plugin

  @State() inProgress: boolean

  componentWillLoad() {
    this.plugin = pluginStore.plugins.entities[this.pluginName]
  }

  private refreshPluginState = () =>
    getAvailablePlugins().then(() => {
      this.plugin = pluginStore.plugins.entities[this.pluginName]
    })

  private install = (name: string) => {
    this.inProgress = true

    window.api
      .invoke(CHANNEL.INSTALL_PLUGIN, name)
      .then(this.refreshPluginState)
      .finally(() => {
        this.inProgress = false
      })
  }

  private uninstall = (name: string) => {
    this.inProgress = true

    window.api
      .invoke(CHANNEL.UNINSTALL_PLUGIN, name)
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
          Upgrade
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
          Uninstall
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
        Install
      </stencila-button>
    )
  }

  render() {
    return (
      <div class="pluginCard">
        <div class="title">
          <span>
            <h2>{this.plugin?.alias ?? this.pluginName}</h2>
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
