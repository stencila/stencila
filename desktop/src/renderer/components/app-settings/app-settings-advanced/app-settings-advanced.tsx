import { Component, h, State } from '@stencil/core'
import { CHANNEL } from '../../../../preload/channels'
import { ConfigSchema } from '../../utils/forms/elements/types'
import { build } from '../../utils/forms/formBuilder'

type Config = Record<string, unknown>
type Settings = {
  config: Config
  schema: ConfigSchema
}

@Component({
  tag: 'app-settings-advanced',
  styleUrl: 'app-settings-advanced.css',
  scoped: true
})
export class AppSettingsAdvanced {
  @State() settings: Settings | undefined

  private readConfig = () =>
    (window.api.invoke(CHANNEL.READ_CONFIG) as unknown) as Promise<Settings>

  async componentWillLoad() {
    this.settings = await this.readConfig()
  }

  render() {
    return (
      <form class="settingsAdvanced">
        {this.settings && build(this.settings.schema)}
      </form>
    )
  }
}
