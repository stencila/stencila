import { Component, h, State } from '@stencil/core'
import { CHANNEL } from '../../../../preload/index'
import { build, ConfigSchema } from '../../forms/formBuilder'

type Config = Record<string, unknown>
type Settings = {
  config: Config
  schema: ConfigSchema
}

@Component({
  tag: 'settings-general',
  styleUrl: 'general.css',
  scoped: true,
})
export class AppSettingsGeneral {
  @State() settings: Settings | undefined

  private readConfig = () =>
    (window.api.invoke(CHANNEL.READ_CONFIG) as unknown) as Promise<Settings>

  async componentWillLoad() {
    this.settings = await this.readConfig()
  }

  render() {
    return (
      <form class="settingsGeneral">
        {this.settings && build(this.settings.schema)}
      </form>
    )
  }
}
