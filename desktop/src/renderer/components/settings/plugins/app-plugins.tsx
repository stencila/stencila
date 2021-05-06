import { Component, h, State } from '@stencil/core'
import { CHANNEL } from '../../../../preload/index'

type Config = Record<string, unknown>
type Plugins = {
  installed: Config
  available: Config
}

@Component({
  tag: 'app-plugins',
  styleUrl: 'app-plugins.css',
  scoped: true,
})
export class AppPlugins {
  @State() plugins: Plugins | undefined

  private getAvailablePlugins = () =>
    (window.api.invoke(
      CHANNEL.LIST_AVAILABLE_PLUGINS
    ) as unknown) as Promise<Plugins>

  async componentWillLoad() {
    this.plugins = await this.getAvailablePlugins()
    console.log(this.plugins)
  }

  render() {
    return (
      <div class="appPlugins">
        <h1>Plugins here</h1>
        <pre>
          <code>{this.plugins && JSON.stringify(this.plugins, null, 2)}</code>
        </pre>
      </div>
    )
  }
}
