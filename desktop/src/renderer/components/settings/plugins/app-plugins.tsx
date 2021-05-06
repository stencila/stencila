import { Component, h, State } from '@stencil/core'
import { CHANNEL } from '../../../../preload/index'
import { getAvailablePlugins, pluginStore } from './pluginStore'

@Component({
  tag: 'app-plugins',
  styleUrl: 'app-plugins.css',
  scoped: true,
})
export class AppPlugins {
  @State() plugins: Plugin[] = []

  @State() inProgress: boolean

  async componentWillLoad() {
    return getAvailablePlugins()
  }

  private checkForUpdates = () => {
    this.inProgress = true

    window.api.invoke(CHANNEL.REFRESH_PLUGINS).finally(() => {
      this.inProgress = false
    })
  }

  render() {
    return (
      <div class="appPlugins">
        <div class="title">
          <h1>Plugins</h1>
          <stencila-button
            onClick={this.checkForUpdates}
            size="xsmall"
            color="neutral"
          >
            Check for updates
          </stencila-button>
        </div>
        {pluginStore.plugins.ids.map((pluginName) => (
          <plugin-card pluginName={pluginName}></plugin-card>
        ))}
      </div>
    )
  }
}
