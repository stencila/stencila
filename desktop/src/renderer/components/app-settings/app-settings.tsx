import { Component, h, State } from '@stencil/core'
import { CHANNEL } from '../../../preload/index'

type Config = Record<string, unknown>
type Settings = {
  config: Config
  plugins: Config[]
}

@Component({
  tag: 'app-settings',
  styleUrl: 'app-settings.css',
  shadow: true,
})
export class AppSettings {
  @State() settings: Settings | undefined

  private readConfig = () =>
    (window.api.invoke(CHANNEL.READ_CONFIG) as unknown) as Promise<Settings>

  async componentWillLoad() {
    this.settings = await this.readConfig()
  }

  render() {
    return (
      <div class="app-settings">
        <h1>Settings</h1>

        {this.settings && (
          <div>
            {Object.entries(this.settings.config).map(
              ([settingLabel, value]) => (
                <div class="settingsItem">
                  <label>{settingLabel}</label>
                  <pre>
                    <code>{JSON.stringify(value, null, 2)}</code>
                  </pre>
                </div>
              )
            )}
          </div>
        )}

        <hr />

        <h2>Plugins</h2>

        {this.settings && (
          <ul>
            {this.settings.plugins.map((plugin) => (
              <li>
                <label>{plugin.name}</label>
              </li>
            ))}
          </ul>
        )}
      </div>
    )
  }
}
