import { Component, h, State } from '@stencil/core'
import { getAvailablePlugins, pluginStore } from './pluginStore'

@Component({
  tag: 'app-plugins',
  styleUrl: 'app-plugins.css',
  scoped: true,
})
export class AppPlugins {
  @State() plugins: Plugin[] = []

  async componentWillLoad() {
    return getAvailablePlugins()
  }

  render() {
    return (
      <div class="appPlugins">
        {pluginStore.plugins.ids.map((pluginName) => (
          <plugin-card pluginName={pluginName}></plugin-card>
        ))}
      </div>
    )
  }
}
