import { Component, Config, h, State } from '@stencil/core'
import { JSONSchema7 } from 'stencila/node_modules/@types/json-schema'
import { client } from '../../../client'
import { build } from '../../utils/forms/formBuilder'

type Settings = {
  config: Config
  schemas: JSONSchema7[]
}

@Component({
  tag: 'app-settings-advanced',
  styleUrl: 'app-settings-advanced.css',
  scoped: true,
})
export class AppSettingsAdvanced {
  @State() settings: Settings | undefined

  private readConfig = () =>
    client.config.getAll().then(({ value: settings }) => {
      return settings.global as Settings
    })

  async componentWillLoad() {
    this.settings = await this.readConfig()
  }

  render() {
    return (
      <form class="settingsAdvanced">
        {this.settings && this.settings.schemas.map((schema) => build(schema))}
      </form>
    )
  }
}
